# 定义自己的领域

## 领域是什么

领域定义了你的应用「说什么话」。包括：

- 系统名称叫什么（"法律智囊团" vs "医疗会诊室"）
- Agent 怎么称呼（"律师" vs "医生"）
- 用户怎么称呼（"委托人" vs "患者"）
- 用什么维度给 Agent 分类（法域 vs 科室）
- 综合分析的 Prompt 模板

## 最小 domain.yaml

```yaml
domain:
  name: "my-domain"
  system_name: "我的智囊团"
  agent_noun: "专家"
  user_title: "用户"

dimensions:
  - id: "category"
    label: "领域"
    values: ["技术", "商业", "设计", "运营"]

synthesis:
  template: |
    你是一位系统协调人。
    
    以下{{agent_count}}位{{agent_noun}}对"{{task}}"给出了分析：
    
    {{#agents}}
    ## {{name}}
    {{output}}
    {{/agents}}
    
    请识别各位{{agent_noun}}的共识与分歧，给出综合建议。

trigger_markers:
  single: ["简单", "快速"]
  conference: ["分析", "综合", "多角度"]
  debate: ["对比", "选择", "优缺点"]
  relay: ["步骤", "流程", "方案"]
  learn: ["学习", "解释", "教我"]
  practice: ["执行", "操作"]
```

## 关键字段说明

### domain

| 字段 | 说明 | 示例 |
|------|------|------|
| `name` | 领域标识 | `legal`, `medical`, `finance` |
| `system_name` | 系统在界面上的名称 | `法律智囊团` |
| `agent_noun` | Agent 的称谓 | `律师`, `医生`, `分析师` |
| `user_title` | 用户的称谓 | `委托人`, `患者`, `客户` |

### dimensions

定义 Agent 的多维分类体系。

| 字段 | 说明 |
|------|------|
| `id` | 维度唯一标识，Agent 定义中引用 |
| `label` | 维度在 UI 上的显示名 |
| `values` | 该维度的离散值列表（用于 UI 下拉和归一化） |
| `weight` | Agent 匹配时该维度的权重，默认 1.0 |

### synthesis

`synthesis.template` 使用 Handlebars 语法。可用变量：

| 变量 | 说明 |
|------|------|
| `{{agent_count}}` | Agent 数量 |
| `{{agent_noun}}` | Agent 称谓（来自 domain.agent_noun） |
| `{{task}}` | 用户输入的任务描述 |
| `{{#agents}}...{{/agents}}` | 遍历所有 Agent 输出 |
| `{{name}}` | Agent 名称 |
| `{{output}}` | Agent 的完整输出 |
| `{{contradictions}}` | 碰撞检测发现的矛盾列表 |
| `{{complements}}` | 碰撞检测发现的互补点 |
| `{{blindspots}}` | 碰撞检测发现的盲区 |

### trigger_markers

控制 Triage 自动分发的关键词。系统会检查用户输入是否包含这些词来决定用哪个推理模式。

## Agent 定义中的维度引用

Agent 通过 `dimensions` 字段关联领域维度：

```yaml
dimensions:
  category: "技术"      # 对应 domain.yaml 中 id="category" 的第四个值
```

框架自动将枚举值转换为数值向量用于距离计算。

## 热切换

```bash
# 方法 1：替换文件后重启
cp my-new-domain.yaml config/domain.yaml
# 重启服务

# 方法 2：通过 API 热切换（无需重启）
curl -X POST http://localhost:3001/api/v1/config/domain \
  -H "Content-Type: application/json" \
  -d '{"domain": "my-new-domain"}'
```

前端会自动检测并更新术语显示。
