# NFR Design Patterns — B3: AI Gateway

## Pattern 1: Gateway Trait + Registry (Provider 抽象)

**问题**: 如何统一 Claude/OpenAI/DeepSeek 三个不同 API 的调用接口？

**方案**: `Gateway` trait + `GatewayRegistry` 注册中心。

```
trait Gateway: Send + Sync {
    fn provider(&self) -> Provider;
    fn is_available(&self) -> bool;
    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>>;
}

GatewayRegistry {
    providers: HashMap<Provider, Arc<dyn Gateway>>
}
```

**规则**:
- 每个 provider 实现 `Gateway` trait
- `GatewayRegistry` 按 `Provider` 枚举路由到具体实现
- 新增 provider 只需实现 trait + 注册，无需修改调用方代码

## Pattern 2: mpsc Channel Streaming (流式响应)

**问题**: LLM 流式响应如何高效传递给上层（B4 Possession Engine）？

**方案**: `tokio::sync::mpsc::unbounded_channel()`。

```
call(prompt, config) -> mpsc::UnboundedReceiver<Result<Chunk>>
  1. 创建 (tx, rx) = mpsc::unbounded_channel()
  2. tokio::spawn(async {
       POST provider API with stream=true
       for each SSE event:
         parse chunk → tx.send(Ok(chunk))
       tx closed on completion/error
     })
  3. return rx
```

**优势**:
- 调用方无需等待完整响应，逐 chunk 消费
- Backpressure: `unbounded_channel` 适合低延迟场景（chunk 通常 < 100 tokens）
- 调用方 drop receiver → channel 关闭 → task 检测并退出

## Pattern 3: Task Isolation (tokio::spawn)

**问题**: 如何并行调用多个 LLM 且支持独立取消？

**方案**: 每个 LLM 调用 spawn 独立 tokio task。

```
call_parallel(requests: &[LLMRequest]) -> Vec<(Provider, JoinHandle<mpsc::Receiver>)>
```

**取消机制**:
```
let handle = tokio::spawn(claude_client.call(prompt, config));
// 超时取消
tokio::select! {
    result = handle => { ... }
    _ = tokio::time::sleep(Duration::from_secs(120)) => {
        handle.abort();
    }
}
```

## Pattern 4: API Key from Environment (环境变量配置)

**问题**: API Key 如何安全管理？

**方案**: 每个 provider client 在 `new()` 时从环境变量读取。

```
impl ClaudeClient {
    pub fn new() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-6".into());
        ClaudeClient { api_key, model, client: reqwest::Client::new() }
    }
}
```

**规则**:
- Key 缺失时 `is_available()` 返回 false
- Key 仅在 `new()` 时读取一次，不在运行时动态变更
- Key 绝不写入日志（`tracing` 不包含 Key 字段）

## Pattern 5: Graceful Provider Degradation (Provider 降级)

**问题**: 部分 provider 未配置时如何保证服务可用？

**方案**: GatewayRegistry 只注册已配置的 provider。

```
GatewayRegistry::new():
  let mut providers = HashMap::new();
  if let Some(client) = ClaudeClient::new().ok() {
      providers.insert(Provider::Claude, Arc::new(client));
  }
  if let Some(client) = OpenAIClient::new().ok() {
      providers.insert(Provider::OpenAI, Arc::new(client));
  }
  if let Some(client) = DeepSeekClient::new().ok() {
      providers.insert(Provider::DeepSeek, Arc::new(client));
  }

list_providers():
  返回每个已注册 provider 的 ProviderInfo { available: true }
  返回每个未注册 provider 的 ProviderInfo { available: false }
```

## Pattern 6: Prompt Template Rendering (模板渲染)

**问题**: 如何将魂数据注入 prompt 模板？

**方案**: Tera 模板引擎，模板文件与代码分离。

```
PromptBuilder::new():
  1. 从 src/prompts/ 目录加载所有 .tera 模板
  2. Tera::new("src/prompts/**/*.tera")

build_summon_prompt(soul, task) -> Prompt:
  1. let mut ctx = Context::new();
  2. ctx.insert("soul", &serialize_soul(soul));
  3. ctx.insert("task", &task);
  4. let rendered = tera.render("summon_prompt.tera", &ctx)?;
  5. parse rendered → Prompt { messages }
```

**模板文件**: `rust/ai-gateway/src/prompts/summon_prompt.tera`
```
[
  {"role": "system", "content": "你是{{ soul.name }}...ismism: {{ soul.ismism_code }}"},
  {"role": "user", "content": "{{ task }}"}
]
```
