# Performance Test Instructions

## Purpose
验证本地单用户场景下的 API 性能基准。

## Performance Requirements (from NFR)

| 指标 | 目标 |
|------|------|
| 请求延迟 P50 | < 10ms |
| 请求延迟 P99 | < 100ms |
| `/api/v1/possess` 响应 | < 50ms |
| HTTP 超时 | 30s |
| WS 空闲超时 | 300s |

## Performance Test

### Local Benchmark (使用 `wrk` 或 `oha`)

```bash
# 安装 oha
brew install oha
# 或: cargo install oha

# 启动 API server
cargo run -p api &
sleep 2

# Health check benchmark
oha -n 1000 -c 10 http://127.0.0.1:3096/api/v1/health

# List souls benchmark (initial empty)
oha -n 100 -c 5 http://127.0.0.1:3096/api/v1/souls

# Cleanup
kill %1
```

### Expected Results (local loopback)

| Endpoint | P50 | P99 |
|----------|-----|-----|
| `GET /health` | < 1ms | < 5ms |
| `GET /souls` (empty) | < 5ms | < 20ms |
| `GET /sessions` (empty) | < 5ms | < 20ms |
| `GET /analytics/mode-distribution` | < 10ms | < 30ms |

## Notes

- 本项目为本地单用户应用（绑定 127.0.0.1），无高并发场景
- LLM 调用（possess）延迟取决于外部 API provider，不在本地性能测试范围内
- WS 连接数 ≤ 10，不需要压力测试
