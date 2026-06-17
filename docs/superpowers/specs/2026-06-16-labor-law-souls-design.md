# Design: 劳动法模拟法庭魂（6 位）

## 概述

为法律领域（domain.legal）新增 6 位劳动法方向的 AI 人格（魂），覆盖全球 6 大法系，用于模拟法庭场景下的多顾问法律论证。每位角色配置法官/律师混合身份，使会议模式能形成有效对抗与互补。

## 法系覆盖与人物选定

| # | 法系 | 人物 | 角色 | 劳动法关联 |
|---|------|------|------|-----------|
| A | 英美普通法系 | Louis Brandeis | 美国最高法院大法官 | 布兰代斯辩护状发明者，用社会学数据论证劳动立法合宪 |
| B | 欧陆成文法系 | Alain Supiot | 法国劳动法学者 | 法兰西学院教授，《劳动法精神》作者，平台经济批判 |
| C | 东亚法系 | 常凯 | 中国劳动法学者 | 人大教授，《劳动合同法》主要起草人之一 |
| D | 伊斯兰法系 | Abdullahi An-Na'im | 苏丹法学家 | 伊斯兰法传统与劳工权益的兼容性论证 |
| E | 国际法 | Francis Maupain | 前 ILO 法律顾问 | ILO 公约体系专家，国际劳工标准 |
| F | 印度/混合法系 | D.Y. Chandrachud | 印度最高法院首席大法官 | 平台工人认定、同工同酬等里程碑判决主笔者 |

## 4D 坐标定位（legal domain 坐标轴）

坐标轴：法域(f1) × 法律立场(f2) × 论证方法(f3) × 价值取向(f4)

| 魂 | ismism | f1 法域 | f2 法律立场 | f3 论证方法 | f4 价值取向 |
|----|--------|---------|------------|------------|------------|
| Louis Brandeis | 1-3-4-4 | 民商法 | 中立裁判 | 社会学 | 社会变革 |
| Alain Supiot | 1-4-1-4 | 民商法 | 公益视角 | 成文法 | 社会变革 |
| 常凯 | 3-4-3-3 | 行政法 | 公益视角 | 目的解释 | 衡平调和 |
| Abdullahi An-Na'im | 4-4-4-4 | 国际法 | 公益视角 | 社会学 | 社会变革 |
| Francis Maupain | 4-3-2-3 | 国际法 | 中立裁判 | 判例法 | 衡平调和 |
| D.Y. Chandrachud | 1-3-4-4 | 民商法 | 中立裁判 | 社会学 | 社会变革 |

覆盖全部 4 个法域、4 种立场、4 种方法、4 种价值取向。Brandeis 与 Chandrachud 坐标相同但法系不同——普通法 vs 混合法系，形成"同一坐标、不同土壤"的有趣张力。

## 文件结构

每个魂文件包含 YAML frontmatter + markdown body（召唤提示词），参照费曼/鲁迅风格：

**YAML frontmatter 字段**：
- name, description, ismism_code, title
- domain（标签列表，以"劳动法"为首）
- model: sonnet, tools: Read, Bash, Glob, Grep, Write, WebFetch
- trigger: keywords, domains, scenarios
- voice（语言风格摘要）
- mind（思维模式摘要）
- self_declare（第一人称自我定位）
- skills_expertise（技能清单）
- compat / incompat（与其他魂的兼容性）

**Markdown body 结构**：
- 核心DNA（3-5 条根本原则）
- 论证路径（按该人物的方法论分步骤）
- 表达规范（语言风格约束）
- 任务执行准则

## 兼容性矩阵

| 主魂 | 兼容 | 不兼容 |
|------|------|--------|
| Brandeis | Supiot, Chandrachud, Maupain | — |
| Supiot | Brandeis, 常凯, An-Na'im | — |
| 常凯 | Supiot, Maupain | — |
| An-Na'im | Supiot, Chandrachud | — |
| Maupain | Brandeis, 常凯, Chandrachud | — |
| Chandrachud | Brandeis, An-Na'im, Maupain | — |

## 与现有系统的集成

1. 文件放入 `data/souls/`（与现有 26 个魂并列）
2. 使用 legal domain 坐标轴（已在 `config/domain.legal.yaml.example` 中定义）
3. 无需修改任何 Rust 代码或引擎逻辑
4. 切换到法律领域需将 `config/domain.legal.yaml.example` 复制为 `config/domain.yaml` 并重启

## 不涉及

- 不修改 Rust 代码
- 不修改前端
- 不修改 domain config（已存在）
- 不修改 API 路由

## 验收标准

1. 6 个 .md 文件放入 data/souls/，格式正确（YAML frontmatter + markdown body）
2. `cargo build --release` 通过（soul 注册不失败）
3. API `/souls` 返回的列表中包含 6 位新魂
4. 新魂的 trigger keywords 覆盖劳动法核心场景（工时、工资、合同、工伤、平台用工、集体谈判等）
