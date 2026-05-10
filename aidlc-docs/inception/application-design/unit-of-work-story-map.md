# Unit of Work Story Map — 万民幡 Web Application

## Requirements → Units Mapping

| FR | Requirement | Unit(s) |
|----|-------------|---------|
| FR1.1 | 魂列表浏览 + ismism 筛选 | B2, F2 |
| FR1.2 | 魂详情页 | B2, F2 |
| FR1.3 | 魂搜索（关键词/编码/品级） | B2, F2 |
| FR1.4 | Registry 数据源 | B1, B2 |
| FR2.1 | 单魂附体 | B3, B4, B6, F3 |
| FR2.2 | 多魂合议 + 辩证综合 | B3, B4, B6, F3 |
| FR2.3 | 魂间辩论 + 裁决 | B3, B4, B6, F3 |
| FR2.4 | 魂链接力 | B3, B4, B6, F3 |
| FR2.5 | 使用者学习模式 | B3, B4, B6, F3 |
| FR2.6 | 实践开口 P1-P4 | B3, B4, B5, B6, F3 |
| FR3.1 | 收魂向导 | B3, B6, F2 |
| FR3.2 | 炼化流程 | B2, B3, B6, F2 |
| FR3.3 | 审查流程 | B2, B3, B6, F2 |
| FR3.4 | 自定义角色创建 | B2, B6, F2 |
| FR3.5 | 魂魄管理（升级/散魂） | B2, B5, B6, F2 |
| FR4.1 | 对话自动存档 | B5 |
| FR4.2 | call-records 写入查询 | B5 |
| FR4.3 | 对话历史浏览 | B5, B6, F4 |
| FR5.1 | 召唤统计面板 | B5, F4 |
| FR5.2 | 未召唤魂检测 | B5, F4 |
| FR5.3 | 低效检测告警 | B5, F4 |
| FR5.4 | 配额速率限制 | B3, B5 |
| FR6.1 | 附体后追问 | F3 |
| FR6.2 | 自我否定环节 | F3 |
| FR6.3 | 空椅子环节 | F3 |

## Unit → Requirement Coverage

| Unit | Covers FRs |
|------|-----------|
| B1 Foundation | FR1.4 (data models) |
| B2 Soul Registry | FR1.1-1.4, FR3.2-3.5 |
| B3 AI Gateway | FR2.1-2.6, FR3.1-3.3, FR5.4 |
| B4 Possession Core | FR2.1-2.6 |
| B5 Archive & Analytics | FR2.6(P3), FR3.5, FR4.1-4.2, FR5.1-5.4 |
| B6 API Layer | FR2.1-2.6, FR3.1-3.5, FR4.3 |
| F1 App Shell | — (infrastructure) |
| F2 Soul Browser | FR1.1-1.3, FR3.1-3.5 |
| F3 Possession UI | FR2.1-2.6, FR6.1-6.3 |
| F4 Dashboard | FR4.3, FR5.1-5.3 |

## Mode → Unit Mapping

| Mode | Backend | Frontend |
|------|---------|----------|
| 单魂附体 | B4(single) | F3(single page) |
| 合议 | B4(conference) | F3(conference grid) |
| 辩论 | B4(debate) | F3(debate view) |
| 接力 | B4(relay) | F3(relay view) |
| 学习 | B4(learning) | F3(learning view) |
| 实践开口 | B4(practice_opening) | F3(practice wizard) |

## Data Import Strategy

魂数据从现有万民幡导入：
- `registry.yaml` → B2 (解析 + 写入 SQLite 索引)
- `souls/{name}.yaml` → B1 (FS 复制到 `data/souls/`)
- `call-records.yaml` → B5 (解析 + 写入 SQLite)
- 导入脚本：`scripts/import-souls.sh`
