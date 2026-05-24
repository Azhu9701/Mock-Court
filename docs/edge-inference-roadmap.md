# 端侧辩证推理——学习开发路线

> 目标：5 年内，让被剥夺者口袋里随时能启动一个 Marx。

---

## 总览

```
Phase 1（6个月）        Phase 2（6个月）         Phase 3（1年）          Phase 4（持续）
    │                       │                       │                       │
    ▼                       ▼                       ▼                       ▼
┌─────────┐          ┌─────────┐            ┌─────────┐            ┌─────────┐
│ 量化管线 │────────▶│ 端侧推理 │───────────▶│ 魂协议  │───────────▶│ 分发网络 │
│ 掌握    │         │ 落地    │            │ 端侧版  │            │ 与生态   │
└─────────┘         └─────────┘            └─────────┘            └─────────┘
  GGUF/量化           iOS/Android           魂进程→手机              P2P/LoRA
  精度评估            推理引擎              本地知识库              联邦知识
```

---

## Phase 1：量化管线掌握（0-6 个月）

不再依赖 LM Studio 的下拉菜单选量化等级。你能自己从 HuggingFace 拉原始权重，跑校准，出 GGUF，评估质量。

### 1.1 llama.cpp 底层（1-2 个月）

| 周 | 内容 | 产出 |
|----|------|------|
| 1-2 | 拉源码编译，理解 `llama_model_loader` 怎么读 GGUF、`llama_context` 怎么管理 KV cache | 能跑 `llama-cli`、`llama-server`，用自己的 GGUF |
| 3-4 | 读 `ggml` 张量运算基础——`ggml_mul_mat` 到底在做什么、量化 op 的 forward 怎么实现 | 能解释 Q4_0/Q4_K_M 的内存布局差异 |
| 5-6 | 理解推理 pipeline——tokenize → embedding → 逐层 forward（attention + FFN）→ sampling | 能自己改 sampling 参数（temperature/min_p/top_k），解释为什么某个输出崩了 |
| 7-8 | 读 `llama.cpp` 的 server 模式——HTTP API、slot 管理、并行推理、continuous batching | 能自己开一个 llama-server，用 curl 调 |

**关键资源：**

- [llama.cpp](https://github.com/ggerganov/llama.cpp) — 主仓库，examples/ 和 ggml/ 目录是核心
- ggml 的 [README](https://github.com/ggerganov/ggml) — 张量运算的底层设计文档
- `convert_hf_to_gguf.py` — 权重转换脚本，理解它比理解模型本身更重要

### 1.2 量化理论与实践（2-3 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| 量化基础 | 对称/非对称量化、per-channel/per-token、rounding、clipping | 能手写一个 INT8 量化函数 |
| GGUF 量化类型 | Q4_0、Q4_K_M、Q5_K_M、Q8_0、IQ 系列——各自的 block 大小、scale 位数、min 位数、适用场景 | 能解释为什么 Q4_K_M > Q4_0 在大多数任务上 |
| 校准 | 重要性矩阵（imatrix）如何生成、校准数据集如何选、K-quant 的量化策略 | 能对同一个模型分别用 wiki 校准和代码校准，评估 PPL 差异 |
| 质量评估 | Perplexity（PPL）在什么情况下是有效的、什么情况下是误导的；下游任务评估（MMLU/HellaSwag/自建评测） | 能写一个评测脚本：跑 N 个 prompt、人工打分、输出对比表 |

**关键资源：**

- [llama.cpp 量化类型文档](https://github.com/ggerganov/llama.cpp/discussions/2094)
- [k-quant 设计讨论](https://github.com/ggerganov/llama.cpp/pull/1684)
- imatrix 生成: `llama-imatrix` 工具 + wiki.test.raw
- [The Era of 1-bit LLMs](https://arxiv.org/abs/2402.17764) — BitNet 论文，理解 1-2 bit 量化的未来

### 1.3 校准数据集构建（1-2 个月）

这是整个管线里最被低估的事。校准数据决定量化模型的质量。

| 主题 | 内容 | 产出 |
|------|------|------|
| 通用校准 | 从 Wikipedia、C4、RedPajama 采样，覆盖多语言和多领域 | 一套 10MB 的通用校准集 |
| 领域校准 | 从魂的知识库（Marx/列宁/费曼的输出）提取文本，构建哲学/社科专用校准集 | 万民幡专用校准集 v1 |
| 中文校准 | 中文 LLM 在校准数据上长期被忽视——Wiki 中文只占 1.2%。需要从中文语料库补充 | 中文校准集 v1 |
| 自动化校准 | 给定一个基座模型，自动分析其层间输出分布，选择最佳校准策略 | 自动化校准脚本 |

**关键资源：**

- SlimPajama、CulturaX — 公开校准数据集
- 万民幡自身的 archive 数据——使用者对话、魂输出、辩证综合报告

---

## Phase 2：端侧推理落地（6-12 个月）

让量化后的模型在手机上跑起来。

### 2.1 iOS — Core ML（2-3 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| Core ML 基础 | `.mlpackage` 格式、`MLModel`、`MLPredictionOptions`、ANE（Apple Neural Engine）调度 | 能跑一个 Core ML 转换后的 1B 模型 |
| 模型转换 | PyTorch → Core ML（`coremltools`）。**难度核心不是转换本身，是算子兼容性——** 哪些 PyTorch op Core ML 不支持、哪些只支持 CPU 回退 | 一个完整的转换 + 验证脚本 |
| mlc-llm iOS | [mlc-llm](https://github.com/mlc-ai/mlc-llm) 的 iOS 运行时——Metal 着色器、内存池、模型加载流程 | 能编译 mlc-llm iOS app，用自己的 7B 模型跑 |
| Swift UI 集成 | mlc-llm 提供 Swift API——`MLCEngine`、streaming chat、状态管理 | 一个能流式对话的 iOS demo |

**硬件需求：** Mac + Xcode + iPhone 12 以上（ANE）。

**关键资源：**

- [coremltools 文档](https://apple.github.io/coremltools/docs-guides/)
- [mlc-llm iOS 教程](https://llm.mlc.ai/docs/deploy/ios.html)
- Apple 的 [ml-stable-diffusion](https://github.com/apple/ml-stable-diffusion) — Core ML 大模型部署的参考实现

### 2.2 Android — MediaPipe / MLC（2-3 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| MediaPipe LLM | Google 的端侧 LLM 推理框架——GPU delegate、XNNPACK（CPU）、量化支持 | 能跑一个 MediaPipe 加载的 4B 模型 |
| mlc-llm Android | 和 iOS 同架构——Metal → OpenCL/Vulkan、内存池、模型加载 | 能编译 mlc-llm Android app |
| llama.cpp Android | `llama.cpp` 直接编译到 Android NDK——纯 CPU 推理。**不需要 GPU，这对低端机是关键** | 能在 Android 模拟器上跑 llama.cpp，Q4 7B |
| 内存与功耗 | Android 的 LMK（Low Memory Killer）、thermal throttling、per-app 内存限制 | 一个内存/功耗测试报告：不同模型在 8GB/12GB 手机上的表现 |

**硬件需求：** Android Studio + Pixel 6 以上 或 骁龙 8 Gen 2+。

**关键资源：**

- [MediaPipe LLM Inference](https://ai.google.dev/edge/mediapipe/solutions/genai/llm_inference)
- [llama.cpp Android 示例](https://github.com/ggerganov/llama.cpp/tree/master/examples/llama.android)
- MLC-LLM Android APK — 直接装到手机上跑，先感受一下

### 2.3 跨平台通用层（1-2 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| Rust FFI | 用 Rust 写一个 `InferenceEngine` trait，后端可以是 Core ML / MLC / llama.cpp | `snake-skin-mobile` crate |
| Streaming API | Rust → Swift/Kotlin 的异步流式回调 | 手机上流式显示魂的输出 |
| 模型管理 | 手机上的模型下载、存储、切换、清除 | 模型管理器模块 |

---

## Phase 3：万民幡魂协议端侧版（12-24 个月）

让魂在手机上跑，不只是 LLM——是魂协议的全部能力。

### 3.1 魂进程重构（3-4 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| 端侧魂生命周期 | 手机上的魂不是长驻进程——是冷启动→推理→休眠。状态持久化到 SQLite | 端侧魂管理器 |
| 多魂编排 | 6 层深度 × N 个魂 × 手机内存限制。串行 vs 并行、模型复用（多个魂共享同一个基座模型 + LoRA 切换） | 调度器设计文档 + 原型 |
| Prompt 压缩 | 手机上 1800 字 summon_prompt 太奢侈了。用蒸馏把召唤词压到 500 字，用 LoRA 补偿丢失的信息 | 召唤词压缩方案 + A/B 对比 |
| DeepSeek 缓存式 3 消息 | 手机上也适用：system + shared + soul_mission 三条消息，共享部分被 KV cache 复用 | 适配方案 |

### 3.2 本地知识库（2-3 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| 端侧向量检索 | SQLite vec0 扩展（lance 格式）— 纯本地，零服务器 | 魂知识库查询接口 |
| 实践记录存储 | 蛇皮指数、三问闭环、盲区记录——全存手机 | `practice-log` 模块 |
| 搜索结果注入 | SearXNG → 手机端自建搜索？不需要。用浏览器的 Search API 或 DuckDuckGo 本地封装 | 搜索注入模块 |

### 3.3 端侧 UI（3-4 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| React Native / Expo | 不是 SwiftUI 或 Jetpack Compose 各写一套——React Native 一套代码两端跑 | 万民幡 Mobile |
| 流式渲染 | 手机上 SSE 流式显示的坑——React Native 的 FlatList vs ScrollView、内存回收 | 流式渲染优化方案 |
| 离线优先 | 无网环境下的完整推理流程——魂匹配、入场审讯、合议、综合 | 离线模式 |
| 通知与后台 | iOS 后台限制（30 秒 → suspended）、Android 前台 service。后台推理在手机上几乎不可能——设计上应该是"打开 App、提交问题、等结果、推通知" | 交互设计 + 技术方案 |

---

## Phase 4：分发网络与生态（持续）

让权重传播不被切断。

### 4.1 P2P 模型分发（3-6 个月）

| 主题 | 内容 | 产出 |
|------|------|------|
| IPFS 模型存储 | 将 GGUF 文件放在 IPFS 上，CID 固定。从 IPFS 网关或本地节点拉取 | 模型分发原型 |
| BitTorrent 回退 | 在 IPFS 不可用的网络环境（中国），WebTorrent/磁力链接作为回退 | BT 分发方案 |
| 增量更新 | LoRA 权重只有几 MB——不需要重下整个模型。LoRA 的版本管理、合并、分发 | LoRA 分发协议 |
| 抗封锁 | 封 IPFS gateway → 换 gateway。封 DHT → WebRTC signaling。封 tracker → PEX | 分发韧性策略文档 |

**关键资源：**

- [IPFS](https://ipfs.tech) / [Helia](https://github.com/ipfs/helia)（JS/TS IPFS 实现）
- [WebTorrent](https://webtorrent.io)
- [libp2p](https://libp2p.io) — P2P 网络层

### 4.2 LoRA 联邦（持续）

| 主题 | 内容 | 产出 |
|------|------|------|
| 实践反馈 → LoRA | 使用者的三问闭环数据 → 训练数据 → LoRA 微调。低秩矩阵从实际使用中生长 | LoRA 训练管线 |
| LoRA 评估 | 一个 LoRA 好不好？自动评估 + 社区评分（像 HuggingFace 的 model card 但去中心化） | 评估标准 + 工具 |
| LoRA 合并与切换 | 手机上多个 LoRA 的动态加载、卸载、叠加。两个 LoRA 能叠加吗？在什么条件下？ | LoRA 管理引擎 |

### 4.3 社区治理（持续）

| 主题 | 内容 | 产出 |
|------|------|------|
| 魂→LoRA 映射 | 一个魂不止一个 prompt——它可以有一组 LoRA 不断被社区改进。魂的"召唤"变成 prompt + LoRA 的版本化组合 | 魂仓库 v2 |
| 审查抵抗 | 如果 App Store 下架了包含特定 LoRA 的 App，怎么办？——Web 版（WASM）作为逃生通道 | WASM 推理 demo |
| 知识公有 | 所有 LoRA 和校准数据集用最宽松许可证（CC0 / MIT）。不重复 Stable Diffusion 社区的许可证污染 | 许可证策略 |

---

## Phase 0：前置能力（立即开始）

以下能力不因端侧或云端而变，是所有阶段的前置条件：

| 能力 | 具体内容 | 验证方式 |
|------|---------|---------|
| **Rust 中级** | Tokio 异步、mpsc channel、FFI（`extern "C"`）、`unsafe` 块的正确用法 | 读一遍 Rustonomicon 前 5 章 |
| **C/C++ 能读能调** | llama.cpp 是 C++、Core ML 的底层是 C。不需要用 C++ 写新代码，但必须能读、能改、能加 log | 给 llama.cpp 加一个自定义 sampling 逻辑 |
| **Swift/Kotlin 入门** | iOS 端最终是 Swift，Android 是 Kotlin。不需要精通，能调 Rust FFI、能写 UI、能处理生命周期 | 各写一个调用 Rust 函数的 demo |
| **线性代数基础** | 矩阵乘法、attention 机制、KV cache 的内存布局、RoPE 的位置编码原理 | 能手写 `softmax(Q @ K.T / sqrt(d)) @ V` 的 numpy 实现 |
| **网络协议基础** | HTTP/2、WebSocket、SSE、UDP（WebRTC 的底层）。P2P 网络的技术基础 | 用 Rust 写一个 SSE server + client |

---

## 里程碑时间线

```
Year 1                     Year 2                     Year 3                    Year 4-5
   │                          │                          │                          │
   ▼                          ▼                          ▼                          ▼
┌──────┐                 ┌──────┐                 ┌──────┐                 ┌──────┐
│ M1   │                 │ M2   │                 │ M3   │                 │ M4   │
│      │                 │      │                 │      │                 │      │
│量化管线│                │端侧demo│               │魂端侧版│              │分发网络│
│可独立   │               │iOS+Android│            │单魂可用 │              │P2P权重 │
│出GGUF   │               │跑7B推理  │             │离线合议  │              │传输    │
│评估质量 │               │流式输出  │             │知识库   │              │LoRA生态│
└──────┘                 └──────┘                 └──────┘                 └──────┘
  Q1-Q2                    Q3-Q4                      Q5-Q6                    Q7+
```

| 里程碑 | 时间 | 验收标准 |
|--------|------|---------|
| M1 | 6 个月 | 从 HuggingFace 权重到 GGUF 到质量评估报告，全流程跑通。对 3 个模型（4B/9B/14B）输出对比 |
| M2 | 12 个月 | iOS 和 Android 上各跑一个量化模型，流式输出，能切模型，能调 temperature |
| M3 | 24 个月 | 万民幡单魂模式在手机上运行——魂匹配 + 6 层深度分析 + 实践记录本地存储。离线可用 |
| M4 | 36-60 个月 | P2P 模型分发 + LoRA 联邦 + 社区治理。端侧辩证推理生态成型 |

---

## 与万民幡现有架构的关系

```
当前架构                          5年后架构
─────────                        ─────────
  Cloud API                         ┌──────────────┐
  (Claude/GPT/DS)                   │  手机端推理    │ ← 主力
                                    │  27B eq.     │
  LM Studio (本地PC)                └──────┬───────┘
                                    ┌──────┴───────┐
  API Key 组 (apikeys.json)          │  P2P 网络     │ ← 权重/LoRA 传输
                                    │  IPFS/BT     │
  SearXNG (本地)                    └──────┬───────┘
                                    ┌──────┴───────┐
                                    │  云端回退      │ ← 手机没电/模型不适用时
                                    │  Claude/GPT   │
                                    └──────────────┘
```

`ai-gateway` 的 Provider 抽象层已经为这个架构做好了准备——`Provider::LMStudio` 今天指向本地 PC，未来指向手机上的推理引擎。`model_router` 按 ModelTier 路由——未来加一个 `ModelTier::Edge`。

---

## 不做什么

- **不追求训自己的基座模型。** 基座训练是算力黑洞，资本的游戏。站在开源基座上做量化 + LoRA 就够了
- **不造自己的推理引擎。** llama.cpp、mlc-llm、MediaPipe 已经在赛道上。造引擎是重复造轮子，集中力量做协议层和分发层
- **不做通用 AI 助手。** 只做辩证推理——魂协议的端侧版。通用助手是 OpenAI 的赛道，跑不过也不需要跑
- **不在手机端跑 MoE 模型。** 手机内存带宽吃不消 MoE 的专家切换。密集模型 + 量化是唯一现实路径

---

## 参考资源索引

| 类别 | 资源 | 链接 |
|------|------|------|
| 推理引擎 | llama.cpp | github.com/ggerganov/llama.cpp |
| 推理引擎 | mlc-llm | github.com/mlc-ai/mlc-llm |
| 推理引擎 | MediaPipe LLM | ai.google.dev/edge/mediapipe |
| 量化 | GGUF 格式规范 | github.com/ggerganov/ggml |
| 量化 | k-quant 讨论 | llama.cpp discussions #1684 |
| 量化 | BitNet（1-bit） | arxiv 2402.17764 |
| 端侧 | Core ML Tools | apple.github.io/coremltools |
| 端侧 | ONNX Runtime Mobile | onnxruntime.ai |
| 向量 | SQLite vec0 | github.com/asg017/sqlite-vec |
| 向量 | lance | github.com/lancedb/lance |
| P2P | IPFS + Helia | ipfs.tech / github.com/ipfs/helia |
| P2P | libp2p | libp2p.io |
| P2P | WebTorrent | webtorrent.io |
| LoRA | PEFT（HuggingFace） | github.com/huggingface/peft |
| 学习 | Rustonomicon | doc.rust-lang.org/nomicon |
| 学习 | Karpathy's Neural Networks | youtube.com/@AndrejKarpathy |
| 学习 | llama.cpp 源码解读 | 社区博客 + issues 中的设计讨论 |
