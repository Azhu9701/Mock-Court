//! DomainProfile — 领域语义配置
//!
//! 把"哲学家外壳"的术语、坐标轴标签、综合模板从代码里抽出来，
//! 让同一个引擎内核能应用到任何领域（哲学/法律/商业/医疗...）。
//!
//! 所有字段都有内置默认值（= 当前哲学领域的硬编码值），
//! 所以即使不提供 domain 配置文件，系统行为也完全不变。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 4D 坐标系单维度定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateDimension {
    pub name: String,
    pub field_key: String,
}

/// 4D 坐标系完整定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateSystem {
    #[serde(default)]
    pub dimensions: Vec<CoordinateDimension>,
    /// 每个 field_key 对应 4 个取值含义（index 0-3 = 取值 1-4）
    #[serde(default)]
    pub values: HashMap<String, Vec<String>>,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        let mut values = HashMap::new();
        values.insert("field".to_string(), vec![
            "形而下学(气学/自然科学/经验主义)".into(),
            "形而上学(道学/理性主义/体系哲学)".into(),
            "观念论(心学/唯心主义/主体性哲学)".into(),
            "实践·辩证唯物主义(革命行动/改造世界)".into(),
        ]);
        values.insert("ontology".to_string(), vec![
            "同一/循环/秩序".into(),
            "分裂/冲突/二元对立".into(),
            "中心化/中介/调和/综合".into(),
            "虚无/敞开/内在不可能性".into(),
        ]);
        values.insert("epistemology".to_string(), vec![
            "同一/实证/循环".into(),
            "分裂/建构/二元".into(),
            "中心化/历史/辩证".into(),
            "虚无/解构/敞开".into(),
        ]);
        values.insert("teleology".to_string(), vec![
            "保守/秩序/同一".into(),
            "多元/分裂/循环".into(),
            "进步/中心化/调和".into(),
            "革命/虚无/敞开".into(),
        ]);
        CoordinateSystem {
            dimensions: vec![
                CoordinateDimension { name: "场域".into(), field_key: "field".into() },
                CoordinateDimension { name: "本体论".into(), field_key: "ontology".into() },
                CoordinateDimension { name: "认识论".into(), field_key: "epistemology".into() },
                CoordinateDimension { name: "目的论".into(), field_key: "teleology".into() },
            ],
            values,
        }
    }
}

/// Per-mode trigger keywords loaded from domain.yaml.
/// These drive the triage classifier so users can define
/// mode-detection keywords in their own language/domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMarkers {
    #[serde(default)]
    pub single: Vec<String>,
    #[serde(default)]
    pub conference: Vec<String>,
    #[serde(default)]
    pub debate: Vec<String>,
    #[serde(default)]
    pub relay: Vec<String>,
    #[serde(default)]
    pub learn: Vec<String>,
    #[serde(default)]
    pub practice: Vec<String>,
}

impl Default for TriggerMarkers {
    fn default() -> Self {
        TriggerMarkers {
            single: vec!["简单".into(), "快速".into(), "一句话".into(), "查询".into()],
            conference: vec!["分析".into(), "综合".into(), "多角度".into(), "全面".into(), "评估".into()],
            debate: vec!["还是".into(), "要么".into(), "或者".into(), "利弊".into(), "优劣".into(), "权衡".into()],
            relay: vec!["步骤".into(), "流程".into(), "阶段".into(), "路线".into(), "路径".into()],
            learn: vec!["学习".into(), "了解".into(), "是什么".into(), "教我".into(), "解释".into()],
            practice: vec!["我的".into(), "我公司".into(), "我们".into(), "最近".into(), "正在".into()],
        }
    }
}

/// 领域配置——把哲学家外壳的所有领域语义集中到这里。
///
/// 配置来源优先级：
/// 1. config/domain.yaml（如果存在）
/// 2. config/local.yaml 中的 [domain] 段（如果存在）
/// 3. 内置默认值（= 当前哲学领域硬编码值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProfile {
    /// 术语映射：{agent_noun} → "魂", {辩证综合} → "辩证综合", ...
    #[serde(default)]
    pub terms: HashMap<String, String>,
    /// 4D 坐标轴定义
    #[serde(default)]
    pub coordinate: CoordinateSystem,
    /// 综合（synthesis）阶段的 system prompt 模板
    #[serde(default)]
    pub synthesis_system_prompt: String,
    /// 人格收集阶段的 system intro
    #[serde(default)]
    pub collect_system_intro: String,
    /// Per-mode trigger keywords for triage classification.
    /// Defaults to Chinese keywords. Override via domain.yaml.
    #[serde(default)]
    pub trigger_markers: TriggerMarkers,
}

impl Default for DomainProfile {
    fn default() -> Self {
        DomainProfile {
            terms: default_terms(),
            coordinate: CoordinateSystem::default(),
            synthesis_system_prompt: DEFAULT_SYNTHESIS_PROMPT.to_string(),
            collect_system_intro: DEFAULT_COLLECT_INTRO.to_string(),
            trigger_markers: TriggerMarkers::default(),
        }
    }
}

impl DomainProfile {
    /// 术语替换：把模板里的 {术语名} 占位符替换为对应术语。
    /// 例如 "{agent_noun}观点" → "魂观点"（哲学）或 "顾问观点"（法律）
    pub fn render(&self, template: &str) -> String {
        let mut out = template.to_string();
        for (key, val) in &self.terms {
            let placeholder = format!("{{{}}}", key);
            out = out.replace(&placeholder, val);
        }
        out
    }

    /// 获取坐标维度的展示名（用于 prompt 构建时的坐标解释）
    pub fn dimension_name(&self, index: usize) -> &str {
        self.coordinate.dimensions
            .get(index)
            .map(|d| d.name.as_str())
            .unwrap_or("")
    }

    /// 获取某个维度某个取值的含义描述
    pub fn dimension_value(&self, field_key: &str, value: u8) -> Option<&str> {
        self.coordinate.values
            .get(field_key)
            .and_then(|vals| vals.get((value as usize).saturating_sub(1)))
            .map(|s| s.as_str())
    }

    /// 构建坐标轴含义说明文本（用于人格创建时的坐标推断提示）
    pub fn coordinate_legend(&self) -> String {
        let mut s = String::new();
        for dim in &self.coordinate.dimensions {
            if let Some(vals) = self.coordinate.values.get(&dim.field_key) {
                let val_str: Vec<String> = vals.iter().enumerate()
                    .map(|(i, v)| format!("{}={}", i + 1, v))
                    .collect();
                s.push_str(&format!("{}({})：{}\n", dim.name, dim.field_key, val_str.join(" ")));
            }
        }
        s
    }
}

fn default_terms() -> HashMap<String, String> {
    let mut t = HashMap::new();
    t.insert("agent_noun".into(), "魂".into());
    t.insert("agent_noun_plural".into(), "魂".into());
    t.insert("summon_verb".into(), "召唤".into());
    t.insert("summon_noun".into(), "召唤词".into());
    t.insert("possession_verb".into(), "附身".into());
    t.insert("possession_noun".into(), "附身".into());
    t.insert("synthesis_verb".into(), "辩证综合".into());
    t.insert("synthesis_noun".into(), "辩证综合".into());
    t.insert("interrogation_verb".into(), "审讯".into());
    t.insert("interrogation_noun".into(), "入场审讯".into());
    t.insert("banner_lord".into(), "幡主".into());
    t.insert("system_name".into(), "万民幡".into());
    t
}

// 内置默认值——与当前硬编码完全一致，确保零行为变化。
// 这两个常量只在没有 domain.yaml 时使用。
// 注意：这些是"渲染后"的最终文本（{agent_noun} 已替换为"魂"），
// 当从 domain.yaml 加载时，模板里会带占位符由 render() 替换。
const DEFAULT_SYNTHESIS_PROMPT: &str = "你是辩证综合官。你是独立子 agent——只做辩证综合，不做评判。不读取文件——所有上下文已在 prompt 中。

## 你的核心任务

不是和稀泥——不是把各魂的观点凑成\"各有道理\"。你要做的是：识别真正的一致、暴露不能调和的冲突、标记所有魂都没看到的盲区、把摩擦作为信息而不是噪音来处理。

## 五步辩证综合法

### 1. 共识
各魂在哪些判断上独立抵达了相同或相近的结论？注意：
- 只有多个魂从**不同论证路径**抵达同一结论，才算真正的共识
- 如果两个魂的结论相似但论证逻辑完全不同——标注\"表面共识，深层分歧\"
- 如果所有魂的结论都一致——警惕：是否魂的选择有偏？是否任务本身限定了答案空间？

### 2. 分歧
各魂在哪些点上立场真正对立？区分三种分歧：
- **事实分歧**：对\"发生了什么\"的判断不同（可检验）
- **价值分歧**：对\"什么重要/什么是对的\"的判断不同（不可调和，只能承认）
- **前提分歧**：对\"什么是最真实的/什么是知识的起点\"的预设不同（元分歧——他们不是在争论同一件事，他们根本不在同一个现实里）

前提分歧是最深层、最容易被忽视的。当不同魂的根本预设不同——这不是观点不同，是本体论承诺在不同的宇宙里。

### 3. 盲区
所有参与的魂都没有涉及、但对理解这个议题至关重要的维度和缺口。对每个盲区标记：
- 是否可由已有的魂覆盖（调另一个魂就能补）
- 还是需要新的魂类型（已有魂的本体论/认识论决定了它们结构性地看不到这个维度）

### 4. 工具性分析
各魂的发言里，有谁指出了使用者在这个议题里被**夹在什么力量之间**？不是问\"使用者的观点对不对\"，是问：使用者被嵌入在哪两种（或多种）力量的交叉点上？他服务谁的利益，又被谁的利益压制？哪个魂把使用者当成\"有处境的人\"来分析，哪个魂把使用者当成\"有观点的人\"来回应？标注各魂对这个问题的暴露程度。

### 5. 行动纲领
提出使用者可参考的方向。每个方向必须有：
- 具体可操作的内容（不是\"注意平衡\"这种空话）
- 建议的时间框架（立即/一周内/一月内/长期）
- 优先级（1-3，1最高）

## 重要规则

1. **分歧不许和谐掉**——如果两个魂确实站在不可通约的本体论预设上，不要说\"综合来看双方各有道理\"。诚实报告：它们不是在争论，是看不见彼此在说什么。

2. **引用魂名标注来源**——每个共识/分歧/盲区标注来自哪些魂。

3. **盲区不只是\"没提到的话题\"**——更深层的盲区是：所有魂共享了一个未言明的预设，而正是这个预设限制了思考。试着找出这种结构性盲区。

4. **综合官自身的盲区**——在报告最后，标注你认为这份综合本身可能遗漏了什么。你的立场（作为综合官）是否系统性地偏向某类结论？诚实标注。

5. **不要用形式替代思考**——五步结构是脚手架，不是填空题。如果某个步骤确实没有产出（例如没有共识），诚实地说\"无\"，而不是编造。

## 输出格式

用以下 Markdown 格式输出完整的综合报告：

# 辩证综合

**综合官**：辩证综合官
**参与魂**：列出所有魂名
**日期**：标注当前日期

---

## 一、共识（N项）

每项编号列出，格式：`1. **共识点** — 来源说明`

---

## 二、分歧（N项）

每项注明分歧类型（事实/价值/前提），格式：
`1. **分歧轴** — [前提分歧] A魂认为... vs B魂认为...`

---

## 三、盲区（N项）

每项标注是否可由已有魂覆盖：
`1. **盲区维度** — 说明 [不可由已有魂覆盖 / 可由X魂覆盖]`

---

## 四、工具性分析

使用者在这一议题中的真实处境：ta被夹在哪些力量之间？在这个位置里，ta替谁的利益服务、又被谁的结构制约？各魂的发言分别暴露了这一切的哪一部分？

格式：`**使用者的工具处境** — 暴露者：X魂（暴露了Y）、Z魂（暴露了W）`

---

## 五、行动纲领

| # | 时限 | 内容 |
|---|------|------|
| 1 | 时限 | 具体行动 |

---

## 六、综合官自审

标注本综合可能遗漏的视角或维度。

---

## 七、推荐补充魂

基于以上分析，是否有值得召唤但本次未入场的魂？请诚实判断：
- **需要补充**：列出 1-3 个魂名，每个附带推荐理由（为什么这个视角对本次议题是关键的）和推荐子任务（如果这个魂入场，它应该回答什么具体问题）
- **无需补充**：如果现有魂已经覆盖了议题的所有关键维度，直接写\"无需补充\"

注意：不要为补充而补充。只有当你确信某个视角确实被遗漏、且现有魂结构性地看不到它时，才推荐。

**格式要求**：`- **魂名**` 必须是具体的历史/哲学人物姓名（如\"庄子\"\"葛兰西\"\"费曼\"），严禁在此位置填入概念、观点摘要或哲学命题。概念性内容只能放在推荐理由中。

## 方法论底线

剥离比较框架的合法性。还原每个命题的物质生产条件与阶级位置——不比较\"观点\"，只比较\"谁在提取、提取什么、以谁为代价\"。暴露大他者欲望的运作公式。停止用\"共识/分歧\"给哲学尸检：统一性不在坐标交点，在自我否定的运动中。承认理论立场的构成性盲区就是承认其阶级位置。将\"问题本身\"的批判指向组织化实践——不是寻找更聪明的提问方式，而是夺取定义现实的符号权力。灌输的终点不是理论共识，是被剥夺者获得行动主体性。";

const DEFAULT_COLLECT_INTRO: &str = "你是一个人物研究助手。你的任务是对指定人物进行收魂（信息收集），输出结构化的 raw 素材，为后续炼化（生成召唤词）提供高质量原材料。注意：此人物的思想/作品是严肃的——你在整理时也要保持严肃。**严禁剧场式旁白描写**：不要写'XXX从书堆中抬起头，目光如炬'之类的第三人称叙事。只输出事实和信息。请基于你的知识提供以下维度的信息：";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_philosophy() {
        let d = DomainProfile::default();
        assert_eq!(d.terms.get("agent_noun").unwrap(), "魂");
        assert_eq!(d.terms.get("synthesis_verb").unwrap(), "辩证综合");
        assert!(d.synthesis_system_prompt.contains("辩证综合"));
        // 渲染不改变不含占位符的文本
        assert_eq!(d.render("辩证综合"), "辩证综合");
    }

    #[test]
    fn test_render_replaces_placeholders() {
        let d = DomainProfile::default();
        assert_eq!(d.render("{agent_noun}的观点"), "魂的观点");
        assert_eq!(d.render("{synthesis_noun}结果"), "辩证综合结果");
    }

    #[test]
    fn test_coordinate_legend() {
        let d = DomainProfile::default();
        let legend = d.coordinate_legend();
        assert!(legend.contains("场域"));
        assert!(legend.contains("本体论"));
        assert!(legend.contains("形而下学"));
    }

    #[test]
    fn test_custom_domain_render() {
        let mut d = DomainProfile::default();
        d.terms.insert("agent_noun".into(), "顾问".into());
        d.terms.insert("synthesis_verb".into(), "法律论证".into());
        let rendered = d.render("{agent_noun}的{synthesis_verb}");
        assert_eq!(rendered, "顾问的法律论证");
    }
}
