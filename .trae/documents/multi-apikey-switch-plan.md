# 多 API Key 池管理与一键切换方案

## 目标

借鉴 CC Switch 和 Cherry Studio 的设计模式，在万民幡中实现：
1. **多 API Key 池管理**：每个 Provider（Claude / OpenAI / DeepSeek）支持配置多个 Key
2. **按场景自动路由**：根据模型/魂/任务类型自动选择合适的 Key
3. **一键切换全局 Key**：在设置中快速切换当前启用的 Key

---

## 一、现状分析

### 当前数据模型

```
前端 localStorage:  apikey_claude / apikey_openai / apikey_deepseek → 单一字符串
后端 apikeys.json:  { "anthropic": "sk-xxx", "openai": "sk-yyy", "deepseek": "sk-zzz" }
```

### 当前路由逻辑 (Rust model_router.rs)

```
模型名含 "claude" → 取 anthropic 的 key → Claude API
模型名以 "gpt"/"o" 开头 → 取 openai 的 key → OpenAI API  
其余 → 取 deepseek 的 key → DeepSeek API
```

### 当前设置 UI (settings-dialog.tsx)

- 每个 Provider 一个 Key 输入框
- 保存到 localStorage + 后端 API
- 无多 Key 概念

---

## 二、新数据模型设计

### 前端 TypeScript 类型

```typescript
interface ApiKeyEntry {
  id: string;           // 唯一标识 (uuid)
  provider: Provider;   // "claude" | "openai" | "deepseek"
  label: string;        // 用户自定义标签，如 "个人账号"、"公司账号"、"中转Key"
  key: string;          // API Key 值
  enabled: boolean;     // 是否启用
  isDefault: boolean;   // 是否为该 Provider 的默认 Key
}

type Provider = "claude" | "openai" | "deepseek";
```

### 后端 JSON 存储格式 (apikeys.json)

```json
{
  "keys": [
    {
      "id": "uuid-1",
      "provider": "anthropic",
      "label": "官方Key",
      "key": "sk-ant-xxx",
      "enabled": true,
      "is_default": true
    },
    {
      "id": "uuid-2", 
      "provider": "anthropic",
      "label": "中转Key",
      "key": "sk-transit-xxx",
      "enabled": true,
      "is_default": false
    }
  ],
  "routing_rules": [
    {
      "provider": "deepseek",
      "model_pattern": "claude-*",
      "prefer_key_id": "uuid-5"
    }
  ]
}
```

### 前端 localStorage 存储格式

```
localStorage key: "apikeys"
value: JSON.stringify(ApiKeyEntry[])
```

---

## 三、实现步骤

### 阶段一：后端 Rust 改造

#### Step 1.1 更新数据模型 (`rust/foundation/src/`)

**文件：** `rust/foundation/src/models.rs`（新建或在现有 models 中新增）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub id: String,
    pub provider: String,      // "anthropic" | "openai" | "deepseek"
    pub label: String,
    pub key: String,
    pub enabled: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStore {
    pub keys: Vec<ApiKeyEntry>,
    #[serde(default)]
    pub routing_rules: Vec<RoutingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub provider: String,
    pub model_pattern: String,  // glob 模式，如 "claude-*"
    pub prefer_key_id: String,
}
```

#### Step 1.2 改造 API Key 存储读写 (`rust/api/src/routes/apikey.rs`)

**改造内容：**
- `load_keys()` 从 `HashMap<String, String>` 改为 `ApiKeyStore`
- 数据迁移：首次加载时，如果旧格式存在，自动迁移到新格式
- 新增以下 API 端点：

| 方法 | 路径 | 功能 |
|------|------|------|
| GET | `/api/v1/apikeys` | 列出所有 Key（不返回 key 值，用 `***` 遮掩） |
| POST | `/api/v1/apikeys` | 添加一个新 Key |
| PUT | `/api/v1/apikeys/:id` | 更新指定 Key（label/enabled/is_default） |
| DELETE | `/api/v1/apikeys/:id` | 删除指定 Key |
| POST | `/api/v1/apikeys/:id/set-default` | 将该 Key 设为所在 Provider 的默认 Key |
| POST | `/api/v1/apikeys/verify` | 验证某个 Key 的有效性（发送一个最小请求） |

**保留兼容：** 旧的 `/api/v1/apikey/set` 改为调用新逻辑的别名

#### Step 1.3 改造模型路由 (`rust/ai-gateway/src/model_router.rs`)

**改造内容：**
- `route_to_provider()` 不再只根据模型名路由，增加 routing_rules 匹配
- 路由优先级：
  1. 精确匹配 `routing_rules` 中的 `model_pattern` → 使用指定 `key_id`
  2. 按模型名自动推断 Provider → 使用该 Provider 的 `is_default` Key
  3. 如果默认 Key 不可用 → 尝试同 Provider 的下一个 `enabled` Key（自动故障转移）
- `AiProvider` 枚举增加 `key_id` 字段，用于日志追踪

```rust
pub enum AiProvider {
    DeepSeek { api_key: String, key_id: String },
    OpenAi { api_key: String, key_id: String }, 
    Claude { api_key: String, key_id: String },
}
```

#### Step 1.4 故障转移与重试 (`rust/ai-gateway/src/` 新建 `failover.rs`)

**新建文件：** `rust/ai-gateway/src/failover.rs`

```rust
pub struct FailoverHandler {
    pub max_retries: u32,        // 默认 3
    pub retry_delay_ms: u64,     // 默认 500ms
    pub keys: Vec<ApiKeyEntry>,  // 同 Provider 的可用 Key 列表
}
```

特点：
- 调用失败时自动尝试下一个 enabled Key
- 每个 Key 最多重试 1 次
- 记录失败日志

#### Step 1.5 改造各 Provider 调用函数

- `openai.rs` 的 `call_openai_chat()` / `call_openai_chat_streaming()` 增加 `key_id` 参数
- `claude.rs` 同理
- `deepseek.rs` 同理
- 所有调用方传入 `key_id` 以支持日志追踪

---

### 阶段二：前端 Next.js 改造

#### Step 2.1 创建 API Key 管理数据层

**新建文件：** `nextjs/lib/api-keys.ts`

```typescript
export interface ApiKeyEntry {
  id: string;
  provider: Provider;
  label: string;
  key: string;
  enabled: boolean;
  isDefault: boolean;
}

export type Provider = "claude" | "openai" | "deepseek";

// CRUD 函数
export function loadAllKeys(): ApiKeyEntry[] { ... }
export function saveKeys(keys: ApiKeyEntry[]): void { ... }
export function getDefaultKey(provider: Provider): ApiKeyEntry | undefined { ... }
export function getEnabledKeys(provider: Provider): ApiKeyEntry[] { ... }
```

**数据层功能：**
- 从 localStorage 加载/保存完整的 Key 列表
- 数据迁移：检测旧的 `apikey_claude` 等格式，自动迁移到新格式
- 提供查询方法（按 Provider 筛选、获取默认 Key 等）

#### Step 2.2 改造 Settings Dialog (`settings-dialog.tsx`)

**改造内容：**

原 UI：
```
┌─────────────────────────────────┐
│ Claude (Anthropic)  [_________] │
│ OpenAI              [_________] │
│ DeepSeek            [_________] │
└─────────────────────────────────┘
```

新 UI（参考 CC Switch 的 Provider 卡片设计）：
```
┌─────────────────────────────────────────┐
│ ▼ Claude (Anthropic)                    │
│ ┌─────────────────────────────────────┐ │
│ │ ● 官方Key          sk-ant-***  [编辑]│ │
│ │ ○ 中转Key          sk-tr-***   [编辑]│ │
│ └─────────────────────────────────────┘ │
│ [+ 添加 Key]                            │
├─────────────────────────────────────────┤
│ ▼ OpenAI                                │
│ ┌─────────────────────────────────────┐ │
│ │ ● 默认Key          sk-proj-*** [编辑]│ │
│ └─────────────────────────────────────┘ │
│ [+ 添加 Key]                            │
├─────────────────────────────────────────┤
│ ▼ DeepSeek                              │
│ ...                                     │
└─────────────────────────────────────────┘
```

**交互设计：**
- 每个 Provider 可折叠/展开 Key 列表
- 点击 Key 行前的圆点（●/○）进行**一键切换默认 Key**
- 每个 Key 行右侧有编辑/删除按钮
- 「添加 Key」按钮弹出对话框（label + key 输入框）
- 编辑 Key 时支持「验证连通性」（调用后端 `/api/v1/apikeys/verify`）
- 支持拖拽排序（可选，低优先级）

#### Step 2.3 创建 Key 编辑对话框组件

**新建文件：** `nextjs/components/key-edit-dialog.tsx`

功能：
- 新增/编辑 Key 的表单
- Label 输入（如"个人账号"、"公司Key"）
- Key 值输入（带显示/隐藏切换）
- Provider 选择（新增时）
- 测试连通性按钮
- 保存/取消

#### Step 2.4 改造模型配置中的 Key 选择 (`soul-model-config.tsx`)

**改造内容：**
在魂级别的模型配置中，增加一个可选的「首选 Key」下拉选择器。当魂使用某个模型时，优先使用指定的 Key。

UI 增加：
```
模型：[DeepSeek-V4-Pro       ▼]
推理：[默认                  ▼]
首选Key：[自动（默认Key）    ▼]  ← 新增
```

#### Step 2.5 更新前端 API 调用 (`lib/api.ts`)

- 新增 `fetchApiKeys()` - 获取 Key 列表
- 新增 `addApiKey(entry)` - 添加 Key
- 新增 `updateApiKey(id, partial)` - 更新 Key
- 新增 `deleteApiKey(id)` - 删除 Key
- 新增 `setDefaultKey(id)` - 设置默认 Key
- 新增 `verifyApiKey(id)` - 验证 Key 连通性

#### Step 2.6 改造附体入口 (`possession-entry.tsx`)

**改造内容（最小改动）：**
- 提交时，使用 `getDefaultKey(provider)` 而非直接从 localStorage 取单个 key
- 确保数据层统一从新的 `apikeys` 存储读取

---

### 阶段三：集成与数据迁移

#### Step 3.1 数据迁移脚本

**后端迁移（Rust）：**
- 在 `load_keys()` 启动时自动检测旧格式
- 如果有旧的 `{ "anthropic": "sk-xxx" }` 格式，自动迁移为：
  ```json
  {
    "keys": [
      { "id": "auto-1", "provider": "anthropic", "label": "默认Key", 
        "key": "sk-xxx", "enabled": true, "is_default": true }
    ]
  }
  ```

**前端迁移（TypeScript）：**
- 在 `loadAllKeys()` 中检测 `localStorage` 中是否存在旧的 `apikey_*` 格式
- 如果存在且新格式不存在，自动迁移
- 迁移后保留旧数据作为备份（写入 `apikeys_legacy_backup`）

#### Step 3.2 端到端测试

验证以下流程：
1. 添加多个 Key → 一键切换默认 Key → 发起附体请求 → 确认使用的是切换后的 Key
2. 为某个魂配置特定 Key → 发起该魂的附体 → 确认使用了指定 Key
3. 删除默认 Key → 自动选择下一个 enabled Key 作为新默认
4. 所有 Key 都失败 → 正确的错误提示

---

## 四、文件变更清单

### Rust 后端

| 文件 | 操作 | 说明 |
|------|------|------|
| `rust/foundation/src/models.rs` | 新建 | ApiKeyEntry、ApiKeyStore、RoutingRule 结构体 |
| `rust/api/src/routes/apikey.rs` | 重构 | 多 Key CRUD API，数据迁移 |
| `rust/ai-gateway/src/model_router.rs` | 改造 | 增加 routing_rules 匹配 + Key ID 追踪 |
| `rust/ai-gateway/src/failover.rs` | 新建 | 故障转移与重试逻辑 |
| `rust/ai-gateway/src/openai.rs` | 微调 | 增加 key_id 参数 |
| `rust/ai-gateway/src/claude.rs` | 微调 | 增加 key_id 参数 |
| `rust/ai-gateway/src/deepseek.rs` | 微调 | 增加 key_id 参数 |
| `rust/ai-gateway/src/lib.rs` | 微调 | 导出 failover 模块 |
| `rust/api/src/main.rs` 或路由注册 | 微调 | 注册新路由 |

### Next.js 前端

| 文件 | 操作 | 说明 |
|------|------|------|
| `nextjs/lib/api-keys.ts` | 新建 | API Key 数据层（CRUD + 迁移） |
| `nextjs/components/key-edit-dialog.tsx` | 新建 | Key 新增/编辑对话框 |
| `nextjs/components/settings-dialog.tsx` | 重构 | 多 Key 管理 UI（折叠面板 + 切换） |
| `nextjs/components/soul-model-config.tsx` | 改造 | 增加首选 Key 下拉选择器 |
| `nextjs/components/possession-entry.tsx` | 微调 | 使用新的数据层获取 Key |
| `nextjs/lib/api.ts` | 改造 | 新增 Key 管理相关 API 调用 |
| `nextjs/config/models.ts` | 微调 | 可选的 Provider 与 Key 关联提示 |

---

## 五、路由优先级设计

```
发起 LLM 请求
    │
    ├─ 1. 检查魂级别配置的「首选 Key」
    │     └─ 有指定 → 使用该 Key（如果 enabled）
    │
    ├─ 2. 检查 routing_rules 中的 model_pattern 匹配
    │     └─ 匹配到 → 使用指定的 key_id
    │
    ├─ 3. 根据模型名自动推断 Provider
    │     └─ 使用该 Provider 的 is_default Key
    │
    └─ 4. 故障转移（如果上述 Key 调用失败）
          └─ 遍历同 Provider 的其他 enabled Key 重试
```

---

## 六、UI/UX 参考

参考 CC Switch 的设计语言：
- **卡片式 Provider 列表**：每个 Provider 一张卡片，清晰展示当前启用的 Key
- **一键切换**：点击 Key 前的单选按钮（●）即可切换默认 Key，无需额外确认
- **状态指示**：绿色圆点表示已启用且验证通过，黄色表示未验证，红色表示失败
- **简洁的添加流程**：点击「+ 添加 Key」→ 弹出表单 → 填入 label + key → 保存即生效

---

## 七、实施顺序

1. **Step 1.1-1.2** → 后端数据模型 + API（基础）
2. **Step 1.3-1.5** → 路由改造 + 故障转移
3. **Step 2.1** → 前端数据层
4. **Step 2.2-2.3** → 设置 UI 改造（核心交互）
5. **Step 2.4-2.6** → 模型配置 + 入口改造
6. **Step 3.1-3.2** → 数据迁移 + 端到端验证

每个步骤完成后进行独立验证，确保不引入回归。

---

## 八、注意事项

1. **安全性**：API Key 列表从后端返回时