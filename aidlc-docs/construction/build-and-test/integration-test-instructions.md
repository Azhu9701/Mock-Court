# Integration Test Instructions

## Purpose
验证各 crate 之间的接口调用链路正常工作。

## Test Scenarios

### Scenario 1: Soul Registry → API 全流程
- **Description**: 验证通过 API 创建魂 → 读取魂 → 更新魂 → 删除魂的完整流程
- **Setup**: 启动 API server + 空 data/ 目录
- **Test Steps**:
  1. `POST /api/v1/souls` → 创建新魂
  2. `GET /api/v1/souls` → 列表包含新魂
  3. `GET /api/v1/souls/:name` → 返回完整 profile
  4. `PUT /api/v1/souls/:name` → 更新 grade/field
  5. `DELETE /api/v1/souls/:name` → 204
- **Expected Results**: 5/5 步骤通过
- **Cleanup**: 清理 data/ 目录

### Scenario 2: Possession → WS 流式推送
- **Description**: 验证附体请求 → WS 连接 → 流式消息推送
- **Setup**: 启动 API server + 至少一个魂注册
- **Test Steps**:
  1. `POST /api/v1/possess` → 获取 session_id + ws_url
  2. 连接 `WS /ws/possess/:session_id/main`
  3. 验证收到 SoulChunk/SoulDone 事件序列
- **Expected Results**: WS 连接成功，收到流式消息
- **Status**: 需要 LLM API key 配置

### Scenario 3: Archive → Analytics 查询
- **Description**: 验证导出和统计查询链路
- **Setup**: 有历史 session 数据的 data/ 目录
- **Test Steps**:
  1. `GET /api/v1/sessions` → 返回历史会话列表
  2. `GET /api/v1/sessions/:id` → 返回会话详情含消息
  3. `GET /api/v1/analytics/mode-distribution` → 返回模式分布
  4. `POST /api/v1/archive/export` → 获取 task_id
  5. `GET /api/v1/archive/export/:task_id` → 轮询导出状态
- **Expected Results**: 所有端点返回正确数据结构

### Scenario 4: 错误处理链路
- **Description**: 验证 FoundationError → HTTP 错误映射
- **Test Steps**:
  1. `GET /api/v1/souls/nonexistent` → 404 + `{"error":"..."}`
  2. `POST /api/v1/possess` with empty task → 400 + `{"error":"task is required"}`
  3. `GET /api/v1/nonexistent` → 404
- **Expected Results**: 统一 JSON 错误格式

## Setup Integration Test Environment

### 1. Start API Server
```bash
# 清理测试数据
rm -rf data/

# 启动服务（需要 LLM API key 环境变量才能进行 possess 测试）
cargo run -p api
```

### 2. Health Check
```bash
curl http://127.0.0.1:3096/api/v1/health
# {"status":"ok"}
```

## Run Integration Tests

### Manual Integration Test Script
```bash
#!/bin/bash
BASE="http://127.0.0.1:3096/api/v1"

# Scenario 1: Soul CRUD
curl -s -X POST "$BASE/souls" -H "Content-Type: application/json" \
  -d '{"name":"TestSoul","ismism_code":"1-2-3-3","field":"哲学","ontology":"唯物主义","epistemology":"辩证理性","teleology":"历史进步","grade":"A","domains":["历史"],"tags":["test"],"summon_prompt":"你是一个测试魂"}'

curl -s "$BASE/souls" | jq '.[] | select(.name == "TestSoul")'

curl -s -X PUT "$BASE/souls/TestSoul" -H "Content-Type: application/json" \
  -d '{"grade":"S"}'

curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE/souls/TestSoul"
# 204

# Scenario 4: Error handling
curl -s "$BASE/souls/nonexistent" | jq .
# {"error":"Soul 'nonexistent' not found"}

curl -s -X POST "$BASE/possess" -H "Content-Type: application/json" -d '{}' | jq .
# {"error":"task is required"}
```

## Cleanup
```bash
rm -rf data/
```
