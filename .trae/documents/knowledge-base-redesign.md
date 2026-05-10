# 知识库重设计方案

## 现状诊断

### 数据层面问题
1. **`knowledge_cards` 表是空表** — 表结构完整、CRUD 齐全，但从未被写入。知识卡片以 `role=System, soul_name="知识卡片"` 的 Message 形式混在 messages 表中
2. **FTS5 索引的是 messages 表** — 搜索时返回的是"谁的哪条消息"而不是"这个知识属于哪个问题"
3. **知识卡片内容混在消息流里** — 以 `📇 知识卡片` 前缀标记，与普通魂输出、综合报告混在一起

### 前端层面问题
4. **搜索驱动而非浏览驱动** — 页面打开显示一个搜索框 + 结果列表，没有"图书馆"的感觉
5. **结果无分组** — 同一个问题的多条魂输出 + 综合报告 + 知识卡片平铺展示，杂乱
6. **没有按主题/问题/魂/模式浏览** — 唯一的导航方式是输关键词

### 根因
知识页面被设计成"搜索引擎"，而非"知识库"。用户在万民幡中发起合议/辩论后获得了高质量的多视角分析，但没有任何机制将这些分析组织成可回顾的知识资产。

---

## 设计方案：从搜索引擎到知识图书馆

### 核心思路

```
当前：输入框 → FTS5 搜索 → 混杂结果列表
目标：知识图书馆 → 按主题/卡片/魂浏览 → 搜索辅助
```

万民幡的核心价值是**对一个问题获得多位思想家的多视角分析 + 辩证综合**。知识库应该以"**问题（Session）**"为组织单元，将每个会话沉淀为一个知识条目。

### 页面布局

```
┌─────────────────────────────────────────────────────────┐
│ 知识库                              🔍 全文搜索  [重建]  │
│                                                         │
│ ┌─ Tab: [📋 知识卡片] [📊 分析报告] ───────────────────┐ │
│ │                                                       │ │
│ │  筛选栏: [模式▾] [时间▾] [魂▾]                        │ │
│ │                                                       │ │
│ │  ┌──────────────────────────────────────────────┐     │ │
│ │  │ 📋 如何看待 DeepSeek 拒绝阿里投资，拥抱国资    │     │ │
│ │  │ conference · 2026-05-10 · 马克思 葛兰西 斯大林 邓小平 │   │ │
│ │  │ 核心洞见：国资入主DeepSeek的辩证综合...       │     │ │
│ │  │ [查看详情 →]                                 │     │ │
│ │  └──────────────────────────────────────────────┘     │ │
│ │                                                       │ │
│ │  ┌──────────────────────────────────────────────┐     │ │
│ │  │ 📋 如何看待豆包收费                            │     │ │
│ │  │ conference · 2026-05-10 · 未明子 Aaron 马克思 列宁 │   │ │
│ │  │ 核心洞见：AI服务作为数字生产资料的收费逻辑...  │     │ │
│ │  │ [查看详情 →]                                 │     │ │
│ │  └──────────────────────────────────────────────┘     │ │
│ │                                                       │ │
│ │  ...更多卡片...                                       │ │
│ └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

#### Tab 1：知识卡片（默认）
- 每个卡片对应一个**有综合报告的会话**（conference/debate mode + 已完成）
- 卡片显示：**问题标题** → 参与魂 → 日期 → 卡片摘要（500字）
- 点击跳转到 `/sessions/{id}` 查看完整分析回放
- 支持按模式、时间范围、魂名筛选

#### Tab 2：分析报告
- 每个条目对应一个**会话**，展示完整的综合报告摘要
- 比知识卡片更详细：展示综合报告的第一段（~300字）
- 同样的筛选和跳转能力

#### 搜索栏（始终在顶部）
- 输入关键词后，切换到搜索结果视图（替换 Tab 内容）
- 搜索结果按**会话分组**显示，而非按消息平铺
- 高亮匹配片段，点击跳转到对应会话

---

## 实现步骤

### Step 1：修复数据写入 — 知识卡片入库

**文件**: `rust/possession/src/modes/conference.rs`（第 289-300 行）

**改动**: 在生成知识卡片后，同时写入 `knowledge_cards` 表：
```rust
// 现有代码：存为 Message（保留）
let _ = store.append_message(&card_msg).await;

// 新增：同时写入 knowledge_cards 表
let card_entity = KnowledgeCard {
    id: uuid::Uuid::new_v4().to_string(),
    title: session_title.clone(),
    content: card.clone(),
    source_soul: None,          // 综合产物，不归于单魂
    source_session: Some(session_id.to_string()),
    tags: soul_names.clone(),   // 参与魂名作为标签
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
let _ = store.insert_knowledge_card(&card_entity).await;
```

**文件**: `rust/api/src/routes/archive.rs` — 新增 API 端点：
- `GET /knowledge/cards` — 列表查询（支持 mode/soul/limit/offset 过滤）
- `GET /knowledge/topics` — 获取有综合报告的会话列表作为"知识主题"

**文件**: `rust/api/src/routes/knowledge.rs` — 新增路由：
```rust
.route("/cards", get(list_cards))
.route("/topics", get(list_topics))
```

**文件**: `rust/foundation/src/sqlite.rs` 和 `storage.rs` — 新增查询方法：
- `list_knowledge_topics(mode, limit, offset)` — JOIN sessions + messages 获取有 synthesis 的会话列表

### Step 2：Storage trait 扩展

**文件**: `rust/foundation/src/storage.rs`
```rust
async fn list_knowledge_topics(&self, mode: Option<&str>, limit: usize, offset: usize)
    -> Result<Vec<KnowledgeTopic>>;
```

**文件**: `rust/foundation/src/models.rs`
```rust
pub struct KnowledgeTopic {
    pub session_id: String,
    pub title: String,
    pub mode: String,
    pub created_at: DateTime<Utc>,
    pub soul_names: Vec<String>,     // 参与的魂名列表
    pub card_summary: Option<String>,// 知识卡片摘要（如有）
    pub synthesis_preview: Option<String>, // 综合报告摘要（如有）
}
```

### Step 3：新建前端知识库组件

**文件**: `nextjs/components/knowledge-browser.tsx`（新建）

核心组件，替代当前的 `KnowledgeSearch`：

```
KnowledgeBrowser
├── 搜索栏 (KnowledgeSearchBar)
│   ├── 输入框（防抖搜索）
│   └── 重建索引按钮
├── Tab 栏 (KnowledgeTabs)
│   ├── [知识卡片] [分析报告]
│   └── 筛选栏 (FilterBar): [模式▾] [魂▾] [排序▾]
├── 卡片列表/搜索结果 (KnowledgeCardList)
│   └── KnowledgeTopicCard (每条一个)
│       ├── 标题、模式、日期
│       ├── 参与魂标签
│       ├── 摘要预览
│       └── 点击跳转 /sessions/{id}
└── 模式分布 (ModeBarChart) - 顶部统计
```

**文件**: `nextjs/lib/api.ts` — 新增 API 函数：
```typescript
export interface KnowledgeTopic {
  session_id: string;
  title: string;
  mode: string;
  created_at: string;
  soul_names: string[];
  card_summary: string | null;
  synthesis_preview: string | null;
}

export async function fetchKnowledgeTopics(params: {
  mode?: string; soul?: string; limit?: number; offset?: number
}): Promise<KnowledgeTopic[]>

export async function fetchKnowledgeCards(params: {
  soul?: string; limit?: number; offset?: number
}): Promise<KnowledgeCard[]>
```

**文件**: `nextjs/app/knowledge/page.tsx` — 更新页面：
- 从服务端获取初始数据（SSR 或 $action）
- 引入 `KnowledgeBrowser` 替代 `KnowledgeSearch`

### Step 4：搜索改为按会话分组

**文件**: `rust/foundation/src/sqlite.rs` — 修改 `search_knowledge`：
- 搜索结果按 `session_id` 分组
- 每组返回：session 标题、模式、日期、参与魂、匹配片段数

**文件**: `nextjs/components/knowledge-browser.tsx`：
- 搜索时切换到"搜索结果"视图
- 结果按会话分组展示
- 点击展开查看该会话下的所有匹配消息

### Step 5：清理与移除

- `nextjs/components/knowledge-search.tsx` → 删除（被 `knowledge-browser.tsx` 替代）
- `nextjs/components/mode-bar-chart.tsx` → 保留，在 `KnowledgeBrowser` 中使用

---

## 数据流汇总

```
用户发起附体（提问）
  └─ 各魂分析 → messages 表 (role=Soul)
       └─ 综合官合成 → messages 表 (role=Synthesis)
            └─ 知识卡片提取 → 【新增】knowledge_cards 表
                             → messages 表 (role=System, 保留兼容)

知识库页面
  ├─ Tab1 知识卡片 → GET /knowledge/cards → knowledge_cards 表
  ├─ Tab2 分析报告 → GET /knowledge/topics → sessions + messages JOIN
  └─ 搜索 → GET /knowledge/search → knowledge_fts + sessions JOIN，结果按会话分组
```

---

## 改动文件清单

| 层 | 文件 | 操作 |
|---|---|---|
| Rust possession | `conference.rs#L289-L300` | 新增 `insert_knowledge_card` 调用 |
| Rust API | `knowledge.rs` | 新增 `/cards` `/topics` 两个端点 |
| Rust API | `archive.rs` 或 `store.rs` | knowledge_cards 列表查询方法 |
| Rust foundation | `models.rs` | 新增 `KnowledgeTopic` 结构体 |
| Rust foundation | `sqlite.rs` | 新增 `list_knowledge_topics` 方法 |
| Rust foundation | `storage.rs` | 扩展 Storage trait |
| Next.js | `knowledge/page.tsx` | 重写为服务端组件 + `KnowledgeBrowser` |
| Next.js | `knowledge-browser.tsx` | **新建**核心浏览组件 |
| Next.js | `knowledge-search.tsx` | **删除**（功能合并到 browser） |
| Next.js | `lib/api.ts` | 新增 `KnowledgeTopic` 类型和 API 函数 |
