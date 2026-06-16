# 从现有项目迁移到框架模板

## 适用场景

你已经基于 Snake Skin 原项目做了二次开发，现在想把业务代码迁移到 `snake init` 生成的标准化结构中。

## 迁移步骤

### Step 1：生成标准项目

```bash
snake init my-project --domain custom
```

### Step 2：迁移 Agent 定义

```bash
# 从原项目复制 Agent 文件
cp -r old-project/data/souls/*.md my-project/data/agents/
```

注意事项：
- 原项目用 `SoulProfile` 类型，新框架用 `AgentProfile`（兼容，字段名自动映射）
- 原项目的 `ismism_code` 字段 → 新框架的 `dimensions` 字段
- 如果使用了 ISMISM 四维坐标，维度 ID 映射：
  - `field` → domain.yaml 中 id=`field` 的维度
  - `ontology` → id=`ontology`
  - `epistemology` → id=`epistemology`
  - `teleology` → id=`teleology`

### Step 3：迁移领域配置

```bash
cp old-project/config/domain.yaml my-project/config/domain.yaml
```

验证：
- `dimensions` 中每个维度的 `id` 与 Agent 定义中的 `dimensions` key 一致
- `synthesis.template` 的 Handlebars 语法正确
- `trigger_markers` 覆盖了你的业务关键词

### Step 4：迁移自定义模式

如果写了自定义推理模式：

```bash
cp old-project/rust/possession/src/modes/my_custom_mode.rs \
   my-project/rust/my-agent-app/src/modes/
```

在 `main.rs` 中注册：
```rust
modes::register(PossessionMode::Custom("my_mode"), Arc::new(my_custom_mode::run));
```

### Step 5：迁移自定义工具

Rust 实现的工具：
```bash
cp old-project/rust/possession/src/tools/my_tool.rs \
   my-project/rust/my-agent-app/src/tools/
```

HTTP 声明的工具：
```bash
cp old-project/config/tools.yaml my-project/config/tools.yaml
```

### Step 6：迁移前端页面

如果自定义了页面路由：
```bash
cp old-project/nextjs/app/my-page/page.tsx \
   my-project/nextjs/app/my-page/page.tsx
```

如果自定义了组件：
```bash
cp old-project/nextjs/components/MyComponent.tsx \
   my-project/nextjs/components/MyComponent.tsx
```

### Step 7：验证

```bash
cd my-project
cargo check             # 验证 Rust 编译
cd nextjs && pnpm tsc --noEmit  # 验证 TypeScript
```

### Step 8：清理

```bash
# 删除原项目中留在框架内的业务代码
rm -rf old-project  # 或保留作为备份
```

## 常见兼容性问题

| 问题 | 解决 |
|------|------|
| `SoulProfile` 找不到 | 改用 `AgentProfile`，或添加 `use foundation::models::AgentProfile as SoulProfile;` |
| ISMISM 坐标不匹配 | 在 `domain.yaml` 中定义 ISMISM 四维，Agent 使用相同的维度和值 |
| 前端组件路径变更 | 批量替换 `@/components/soul-` → `@/components/agent-` |
| WsEvent 类型名变更 | `SoulToken` → `AgentToken`（旧名作为别名保留） |
| 硬编码中文术语 | 抽取到 domain.yaml，用 `DomainProfile` 读取 |
