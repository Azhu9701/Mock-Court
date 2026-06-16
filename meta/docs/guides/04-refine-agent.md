# 迭代优化 Agent

## Refine 工作流

```
生产使用 → 收集反馈 → Refine → 生成优化版本 → 人工审核 → 上线
```

## 什么时候用 Refine

| 场景 | 示例反馈 |
|------|---------|
| 覆盖不足 | "对竞品分析的市场数据部分太弱，缺少定量分析" |
| 语气偏离 | "回答太技术化，客户听不懂，需要更通俗" |
| 盲区检测 | "从来没提过隐私合规问题，这在 GDPR 场景下是必须的" |
| 新增能力 | "需要加上数据分析能力，能引用 Statista 数据" |
| 碰撞后优化 | "跟 B Agent 的结论总是矛盾，需要协调整体立场" |

## API 调用

```bash
curl -X POST http://localhost:3001/api/v1/souls/refine \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_TOKEN" \
  -d '{
    "name": "竞品分析师",
    "feedback": "输出缺少竞品的财务数据对比。
                 需要加上营收、市场份额等定量维度。
                 另外语气太保守，需要更直接地指出优劣势。",
    "context": "用于季度商业分析报告",
    "examples": [
      "上次分析Apple时只提了产品功能，没提市场份额",
      "分析SaaS竞品时没对比定价策略"
    ]
  }'
```

**响应**：

```json
{
  "original": { "...原 Agent 配置..." },
  "refined": {
    "name": "竞品分析师",
    "changes": {
      "system_prompt": "更新了系统 Prompt，新增财务数据分析和定价策略维度",
      "tools": ["添加了 web_search 工具"],
      "trigger_keywords": ["添加了 '财报'、'市场份额'、'营收'"]
    },
    "profile": { "...优化后的完整配置..." }
  },
  "diff": "...变更对比 (Markdown)..."
}
```

## 审核后应用

```bash
# 查看 refined 版本
curl http://localhost:3001/api/v1/souls/竞品分析师/refined

# 确认应用
curl -X PUT http://localhost:3001/api/v1/souls/apply-refine \
  -H "Content-Type: application/json" \
  -d '{"name": "竞品分析师"}'
```

## 迭代记录

每次 Refine 都会保存历史版本：

```bash
# 查看修改历史
curl http://localhost:3001/api/v1/souls/竞品分析师/revisions
```

```json
{
  "revisions": [
    {
      "version": 3,
      "reason": "补充财务数据分析和定价策略维度",
      "changed_by": "refine",
      "changed_at": "2026-06-16T10:30:00Z"
    },
    {
      "version": 2,
      "reason": "优化语气，从技术化改为商务化",
      "changed_by": "refine",
      "changed_at": "2026-06-14T15:00:00Z"
    },
    {
      "version": 1,
      "reason": "初始创建",
      "changed_by": "collect",
      "changed_at": "2026-06-10T09:00:00Z"
    }
  ]
}
```

支持回滚到任意历史版本。
