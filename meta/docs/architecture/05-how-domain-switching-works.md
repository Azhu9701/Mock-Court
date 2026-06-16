# domain.yaml 驱动机制

## 设计原理

框架内核不包含任何领域术语（"顾问"、"律师"、"魂"、"工友" 等）。所有领域特定内容由 `config/domain.yaml` 注入，实现**不改代码切换领域**。

## domain.yaml 结构

```yaml
# 示例：法律顾问领域
domain:
  name: "legal"
  icon: "⚖️"
  system_name: "法律智囊团"            # 系统自称
  agent_noun: "顾问"                   # Agent 称谓
  user_title: "委托人"                 # 用户称谓
  synthesis_verb: "综合论证"            # 综合阶段的动作名

dimensions:                            # 坐标维度定义
  - id: "law_area"
    label: "法域"
    description: "所属法律领域"
    values: ["民事", "刑事", "行政", "国际"]
    weight: 1.0
  - id: "position"
    label: "立场"
    description: "法律立场取向"
    values: ["原告方", "被告方", "中立", "公益"]
    weight: 0.8
  - id: "method"
    label: "方法"
    description: "法律论证方法"
    values: ["法条主义", "判例法", "目的论", "社会学"]
    weight: 0.6
  - id: "value"
    label: "取向"
    description: "价值取向"
    values: ["秩序优先", "权利优先", "衡平优先", "变革优先"]
    weight: 0.6

synthesis:                             # 综合 Prompt 模板
  template: |
    你是一位资深{{agent_noun}}协调人。
    
    以下{{agent_count}}位{{agent_noun}}针对"{{task}}"分别给出了分析：
    
    {{#agents}}
    ## {{name}}（{{dimensions}}）
    {{output}}
    {{/agents}}
    
    碰撞检测发现：
    - 矛盾点：{{contradictions}}
    - 互补点：{{complements}}
    - 盲区：{{blindspots}}
    
    请完成{{synthesis_verb}}，给出综合意见。

collect_intro: |                       # Collect 阶段的引导语
  我将帮助{{user_title}}创建一位新的{{agent_noun}}。
  请描述你希望这位{{agent_noun}}具备的专业领域和特点……

trigger_markers:                       # Triage 关键词映射
  single: ["简单", "快速", "一句话"]
  conference: ["分析", "综合", "多角度"]
  debate: ["辩论", "对立", "正方反方"]
  relay: ["步骤", "流程", "阶段"]
  learn: ["学习", "教我", "解释"]
  practice: ["实践", "操作", "执行"]

# Agent 定义文件路径
agent_data_path: "./data/agents/"

# 记忆配置
memory:
  default_share: "session"            # none | session | all
  max_turns: 20
  persist: true
```

## 变量注入流程

```
1. 服务启动
   ├─ foundation::config::load() 读取 config/default.yaml
   └─ 读取 domain.yaml → DomainProfile 结构

2. Prompt 构建（每次 LLM 调用）
   ├─ PromptBuilder 持有 Arc<DomainProfile>
   ├─ build_conference_prompt(task, agents, domain) {
   │     let template = &domain.synthesis.template;
   │     template.replace("{{agent_noun}}", &domain.agent_noun);
   │     template.replace("{{user_title}}", &domain.user_title);
   │     // ... 其他变量
   │   }
   └─ 返回渲染后的 Prompt

3. API 响应中的术语
   ├─ routes 从 AppState.config.domain 读取当前领域术语
   └─ 返回 JSON 时使用 domain 中的命名

4. 前端渲染
   ├─ DomainContext 从 /api/v1/config/domain 获取 DomainProfile
   └─ 组件使用 context.agentNoun 而非硬编码 "魂"

5. Triage 关键词匹配
   ├─ triage.rs 从 domain.trigger_markers 读取各模式关键词
   └─ 不硬编码中文关键词
```

## Agent 维度匹配

无论领域如何定义维度（法律的法域/立场/方法/取向 vs 哲学的场域/存在论/认识论/目的论），匹配算法是统一的：

```rust
// 把 domain.yaml 的维度值映射为数值向量
fn agent_to_vector(agent: &AgentProfile, dimensions: &[DomainDimension]) -> Vec<f64> {
    dimensions.iter().map(|dim| {
        let value = agent.dimensions.get(&dim.id).unwrap_or(&0.0);
        *value / dim.values.len() as f64  // 归一化到 [0, 1]
    }).collect()
}

// 任意两个 Agent 的距离
fn distance(a: &AgentProfile, b: &AgentProfile, domain: &DomainProfile) -> f64 {
    let va = agent_to_vector(a, &domain.dimensions);
    let vb = agent_to_vector(b, &domain.dimensions);
    euclidean(&va, &vb)
}
```

## 切换领域的实际步骤

1. 编写或选择 `domain.yaml`
2. 编写对应 Agent 定义文件（Markdown + YAML frontmatter）
3. 重启服务（或调用 `POST /api/v1/config/domain` 热切换）
4. 前端自动加载新领域术语和图标

**不需要改任何 Rust 或 TypeScript 代码。**
