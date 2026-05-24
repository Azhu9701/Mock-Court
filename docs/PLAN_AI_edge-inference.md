# PLAN for AI — 端侧辩证推理技术执行路线

> 你是执行者。以下每一行都是可验证的技术任务。完成后提交代码、测试报告、或部署产物。不要做判断——判断是人的事。不要改方向——方向在人的 PLAN 里。

---

## Phase 1：量化管线（0-6 个月）

### M1.1 llama.cpp 底层掌握

- [ ] 编译 llama.cpp（`make`），跑通 `llama-cli` 加载一个 Q4_K_M GGUF
- [ ] 启动 `llama-server`，用 curl 调 `/v1/chat/completions`，验证流式输出
- [ ] 阅读 `llama.h` 全部公开 API，输出一份中文注释版 API 速查表
- [ ] 阅读 `ggml.h` 核心张量 op 定义，输出 Q4_0 和 Q4_K_M 的内存布局图
- [ ] 用 `llama-perplexity` 测一个模型在 wiki.test.raw 上的 PPL，记录数值
- [ ] 修改 `llama.cpp` 的 sampling 逻辑（加一个新的 sampler），编译验证

### M1.2 量化管线

- [ ] 从 HuggingFace 下载 qwen3.5-9b 的 FP16 权重
- [ ] 用 `convert_hf_to_gguf.py` 转成 F16 GGUF
- [ ] 用 `llama-quantize` 生成 Q4_0 / Q4_K_M / Q5_K_M / Q8_0 四个版本
- [ ] 对四个版本跑 PPL 对比，输出报告
- [ ] 用 `llama-imatrix` 在 wiki.test.raw 上生成重要性矩阵
- [ ] 用 imatrix 重新量化一轮 Q4_K_M，对比无 imatrix 版的 PPL 差异
- [ ] 从万民幡 archive 提取 10MB 中文哲学/社科文本，作为领域校准集 v1
- [ ] 用领域校准集生成 imatrix，量化模型，评估对比

### M1.3 自动化评测

- [ ] 写一个评测脚本：10 个相同 prompt × 多个量化版本，跑推理，输出对照表
- [ ] 评测维度：字数、L4 辩证反转存在性（关键词检测）、重复率（trigram Jaccard）
- [ ] 对 qwen3.5-9b / qwen3.6-27b / qwen3-4b 三个模型各出 4 个量化版本，全量评测，输出最终对比报告

---

## Phase 2：端侧推理落地（6-12 个月）

### M2.1 iOS — Core ML + mlc-llm

- [ ] 用 `coremltools` 将一个 4B 模型从 PyTorch 转到 Core ML（`.mlpackage`）
- [ ] 列出转换中不支持/CPU 回退的 op，输出兼容性报告
- [ ] 编译 mlc-llm iOS app，用自备的 7B Q4_K_M GGUF 跑通流式对话
- [ ] 写一个最小 SwiftUI demo：选择模型 → 输入文本 → 流式显示回复
- [ ] 实现 Rust → Swift FFI 的最小通路：Rust 编译为 `cdylib`，Swift 通过 C bridge 调用，验证数据来回正确

### M2.2 Android — llama.cpp + MediaPipe

- [ ] 编译 llama.cpp 到 Android NDK（arm64-v8a）
- [ ] 写一个最小 Android demo：加载 4B Q4 GGUF，CPU 推理，显示在 TextView 上
- [ ] 测试 MediaPipe LLM Inference API 加载 4B 模型的 GPU 推理速度，和 CPU 推理对比
- [ ] 测试不同模型大小（4B/7B/9B）在 8GB 手机的推理速度和内存峰值，输出报告
- [ ] 实现 Rust → Kotlin FFI 的最小通路

### M2.3 跨平台 Rust 层

- [ ] 定义 `InferenceEngine` trait：`load(model)` / `infer(prompt) → Stream<Chunk>` / `unload()`
- [ ] 实现 `LlamaCppEngine`（用 `llama.cpp` 的 C API 通过 FFI）
- [ ] 实现 `MlcEngine`（调 mlc-llm 的 C API）
- [ ] 编译到 iOS（`aarch64-apple-ios`）和 Android（`aarch64-linux-android`）各一次，记录所有编译问题和解决方案

---

## Phase 3：万民幡魂协议端侧版（12-24 个月）

### M3.1 魂进程端侧化

- [ ] 用 Rust 实现端侧魂生命周期管理器：cold_start → load_model → infer → save_state → unload
- [ ] 魂状态（实践记录/盲区/会话历史）序列化到 SQLite，支持断点恢复
- [ ] 多魂共享基座模型 + LoRA 动态切换的原理验证：同一模型加载不同 LoRA，对比输出一致性
- [ ] Prompt 压缩：将 1800 字 summon_prompt 蒸馏到 500 字，对比 10 个 prompt 的输出一致性（≥90%）
- [ ] 适配 `build_summon` 的 3 消息缓存格式到端侧 KV cache

### M3.2 本地知识库

- [ ] 集成 sqlite-vec 到 Rust 项目，建表，插入 1000 条魂输出文本，验证向量检索速度
- [ ] 实现实践记录本地 CRUD：蛇皮指数、三问闭环、盲区记录
- [ ] 实现本地搜索封装：不依赖 SearXNG 服务器，用 DuckDuckGo 或浏览器 Search API

### M3.3 端侧 UI

- [ ] 用 React Native（Expo）搭万民幡 Mobile 骨架：魂列表、输入框、流式输出
- [ ] 实现 SSE 流式渲染，对比 FlatList 和 ScrollView 在 200+ 条消息时的性能
- [ ] 实现离线模式：断网 → 魂匹配 → 推理 → 结果显示，全流程可用
- [ ] iOS 后台限制测试：推理中切换到后台，记录存活时间，设计"推通知"方案

---

## Phase 4：分发网络（24 个月+）

### M4.1 P2P 模型分发

- [ ] 将一个 9B Q4 GGUF（~6GB）上传到 IPFS，固定 CID
- [ ] 写一个 Rust/IPFS 下载脚本：从 CID 拉文件、断点续传、校验 hash
- [ ] 实现 WebTorrent 回退方案：同样的文件做磁力链接，验证下载速度
- [ ] 测试中国网络环境下 IPFS 可及性，记录所有被墙的 gateway，找到可用的

### M4.2 LoRA 联邦

- [ ] 从万民幡实践记录中提取 100 组（提问，优质回复）对，作为 LoRA 训练数据
- [ ] 用 PEFT 训练一个小 LoRA（rank=16）到 qwen3.5-9b
- [ ] 写 LoRA 评估脚本：加载 LoRA vs 不加，同一 prompt 集，对比输出并打分
- [ ] 实现 LoRA 热加载/卸载：推理中动态切换 LoRA，验证内存正确释放

### M4.3 WASM 逃生通道

- [ ] 编译 llama.cpp 到 WASM（Emscripten），在浏览器加载 4B 模型
- [ ] 写一个单页 HTML：选模型文件 → 加载 → 输入 prompt → 流式输出
- [ ] 测试 Chrome/Safari/Firefox 的 WASM 内存上限（4GB），确定 WASM 能跑的最大模型

---

## 全阶段持续任务

- [ ] 每个 Phase 完成后，跑一轮完整的滴滴同题评测，对比上个 Phase 的输出质量
- [ ] 评测数据（prompt、输出、评分、字数字）全部存入 archive
- [ ] 每个 M 完成后更新 README 模型对比表
