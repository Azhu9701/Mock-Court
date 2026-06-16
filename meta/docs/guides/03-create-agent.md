# 创建 Agent

Agent 定义文件采用 Markdown + YAML frontmatter 格式。放在 `data/agents/` 目录下，框架自动加载。

## Agent 定义模板

```yaml
---
name: "合同审查员"                      # Agent 唯一名称
title: "合同法专家"                     # 显示标题
description: "专注于商业合同的风险审查"      # 简短描述
model: "claude-3-5-sonnet"             # 推荐模型
tools:                                  # 可用工具列表
  - "search_law"
  - "analyze_contract"
trigger_keywords:                       # 触发关键词（用户输入命中则自动匹配）
  - "合同"
  - "违约"
  - "赔偿"
  - "协议"
dimensions:                             # 领域坐标（对应 domain.yaml 的 dimensions）
  law_area: "民事"
  position: "被告方"
  method: "法条主义"
  value: "秩序优先"
system_prompt: |                        # 系统 Prompt（发送给 LLM）
  你是专注于中国合同法的法律顾问。
  
  分析合同时应关注：
  1. 合同效力（是否满足成立要件）
  2. 关键条款（违约责任、管辖、保密等）
  3. 常见风险点（格式条款、显失公平等）
  4. 履约可行性（交付、验收标准等）
  
  输出结构：
  - 总体风险评估（低/中/高）
  - 关键条款分析
  - 风险点清单
  - 修改建议
compat:                                 # 兼容 Agent（合议时优先匹配，可选）
  - "诉讼策略师"
incompat:                               # 不兼容 Agent（合议时避免，可选）
  - "调解员"
domains:                                # 擅长领域标签（可选）
  - "合同法"
  - "商法"
exclude_scenarios: []                   # 排除场景（可选）
voice: "专业严谨"                       # 语气特征（可选）
mind: "法条主义"                         # 思维模式（可选）
---

# 合同审查员

## 分析路径

1. 先快速浏览合同全文，识别合同类型
2. 逐条审查关键条款
3. 对照相关法规检视合规性
4. 输出风险矩阵

## 常用法规

- 《中华人民共和国民法典》第三编 合同
- 《民法典合同编通则司法解释》
- 相关行业主管部门规章

## 与其他顾问的协作

- 与**诉讼策略师**配合：评估条款在诉讼中的风险
- 与**法规研究员**配合：获取特定行业的最新法规
```

## Collect：从描述自动生成 Agent

如果你不想手写，可以通过 Collect 功能让 AI 帮你生成：

```bash
curl -X POST http://localhost:3001/api/v1/souls/collect \
  -H "Content-Type: application/json" \
  -d '{
    "description": "需要一个专门分析竞品文案的Agent，
                    能从产品定位、营销策略、用户心理三个角度切入，
                    语气犀利，敢于指出问题",
    "domain": "marketing"
  }'
```

AI 会自动生成：
- 名称和标题
- 坐标维度
- 系统 Prompt
- 触发关键词
- 分析路径

生成结果会先返回给你预览，确认后保存。

## Refine：基于反馈迭代优化 Agent

Agent 上线后，你可以给出反馈让它自我改进：

```bash
curl -X POST http://localhost:3001/api/v1/souls/refine \
  -H "Content-Type: application/json" \
  -d '{
    "name": "合同审查员",
    "feedback": "对劳动合同的场景覆盖不足，
                 需要在系统 Prompt 中加入《劳动合同法》相关内容，
                 同时增加竞业限制条款的分析",
    "examples": ["审查了3份劳动合同，都没注意到竞业限制条款的漏洞"]
  }'
```

AI 会基于反馈修改 System Prompt、添加触发关键词、调整分析路径。

## Agent 文件热加载

默认情况下，Agent 定义文件变更需要调用 reload 接口：

```bash
curl -X POST http://localhost:3001/api/v1/souls/reload
```

也可以在 `config/default.yaml` 中开启文件监控自动 reload：

```yaml
watch_agents: true       # 监控 data/agents/ 目录变化
watch_interval: 5        # 扫描间隔（秒）
```
