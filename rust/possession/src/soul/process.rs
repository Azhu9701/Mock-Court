use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing;

use ai_gateway::GatewayRegistry;
use foundation::{LLMRequest, Prompt, PromptMessage, Provider, Result, SoulProfile, UsageStats};

/// 魂进程状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoulProcessState {
    /// 未启动
    Stopped,
    /// 运行中
    Active,
    /// 休眠中
    Sleeping,
}

/// 魂进程事件
#[derive(Debug)]
pub enum SoulProcessEvent {
    /// 发送任务
    Task {
        task: String,
        presets: Option<UserPresets>,
    },
    /// 休眠进程
    Sleep,
    /// 唤醒进程
    Wake,
    /// 停止进程
    Stop,
}

/// 魂进程输出事件
#[derive(Debug, Clone)]
pub enum SoulProcessOutput {
    /// Token 块
    Chunk(String),
    /// 完成
    Done {
        content: String,
        usage: UsageStats,
    },
    /// 被干预中断，携带已生成的部分内容和干预信息
    Interrupted {
        partial_content: String,
        intervention: Intervention,
    },
    /// 错误
    Error(String),
}

/// 运行时干预指令
///
/// 由碰撞检测引擎或综合官向推理中的魂发送，实现推理与干预的竞态。
#[derive(Debug, Clone)]
pub enum Intervention {
    /// 矛盾质疑：发现当前魂的推理与其他魂存在矛盾
    ContradictionQuestion {
        /// 发起干预的魂名
        from_soul: String,
        /// 矛盾描述
        contradiction: String,
        /// 追问问题
        question: String,
    },
    /// 盲点重定向：当前魂的推理被其他魂覆盖，建议转向新方向
    BlindSpotRedirect {
        /// 覆盖当前视角的魂名
        covered_by: String,
        /// 建议的新方向
        suggested_direction: String,
    },
    /// 深化请求：当前魂的推理深度不足，需要深入特定维度
    DeepenRequest {
        /// 需要深化的维度
        aspect: String,
        /// 深化原因
        reason: String,
    },
}

impl Intervention {
    /// 将干预指令转换为可注入上下文的消息
    pub fn to_prompt_message(&self) -> PromptMessage {
        let content = match self {
            Intervention::ContradictionQuestion {
                from_soul,
                contradiction,
                question,
            } => {
                format!(
                    "【运行时干预 — 矛盾质疑】\n\
                     来自魂「{}」的反馈：发现以下矛盾点——\n「{}」\n\n\
                     请重新审视你的推理，并回应以下问题：\n「{}」\n\n\
                     注意：你的原有推理方向与此反馈存在张力，请尝试融合或明确指出分歧的根源。",
                    from_soul, contradiction, question
                )
            }
            Intervention::BlindSpotRedirect {
                covered_by,
                suggested_direction,
            } => {
                format!(
                    "【运行时干预 — 盲点重定向】\n\
                     魂「{}」已经覆盖了你当前的推理角度。\n\n\
                     请转向以下新方向重新推理：\n「{}」\n\n\
                     避免与已覆盖内容重复，寻找独特的贡献角度。",
                    covered_by, suggested_direction
                )
            }
            Intervention::DeepenRequest { aspect, reason } => {
                format!(
                    "【运行时干预 — 深化请求】\n\
                     需要你在「{}」维度上深化分析。\n\n\
                     原因：{}\n\n\
                     请在原推理基础上，深入挖掘该维度的内涵、边界条件和实践意义。",
                    aspect, reason
                )
            }
        };

        PromptMessage {
            role: "user".to_string(),
            content,
            reasoning_content: None,
            ..Default::default()
        }
    }
}

/// 预设信息（简化版）
#[derive(Debug, Clone)]
pub struct UserPresets {
    pub judgment: Option<String>,
    pub worry: Option<String>,
    pub unknown: Option<String>,
}

/// 历史消息记录
#[derive(Debug, Clone)]
pub struct HistoricalMessage {
    pub role: String,
    pub content: String,
    pub reasoning_content: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// 魂进程上下文
#[derive(Debug, Clone)]
pub struct SoulContext {
    /// 魂配置
    pub profile: SoulProfile,
    /// 历史消息
    pub history: VecDeque<HistoricalMessage>,
    /// 最大历史记录数
    pub max_history: usize,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后活动时间
    pub last_active: DateTime<Utc>,
}

impl SoulContext {
    fn new(profile: SoulProfile) -> Self {
        SoulContext {
            profile,
            history: VecDeque::new(),
            max_history: 50,
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }

    fn add_message(&mut self, role: String, content: String, reasoning_content: Option<String>) {
        self.history.push_back(HistoricalMessage {
            role,
            content,
            reasoning_content,
            timestamp: Utc::now(),
        });
        self.last_active = Utc::now();

        // 限制历史消息数量
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    fn build_prompt(&self, task: &str, presets: &UserPresets) -> Prompt {
        let mut messages = vec![PromptMessage {
            role: "system".to_string(),
            content: self.profile.summon_prompt.clone(),
            reasoning_content: None,
            ..Default::default()
        }];

        // 添加历史消息
        for msg in &self.history {
            messages.push(PromptMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
                reasoning_content: msg.reasoning_content.clone(),
                ..Default::default()
            });
        }

        // 添加用户预设
        let mut user_prompt = String::new();
        if let Some(j) = &presets.judgment {
            user_prompt.push_str(&format!("\n【预设判断】{}\n", j));
        }
        if let Some(w) = &presets.worry {
            user_prompt.push_str(&format!("\n【预设担忧】{}\n", w));
        }
        if let Some(u) = &presets.unknown {
            user_prompt.push_str(&format!("\n【预设未知】{}\n", u));
        }
        user_prompt.push_str(&format!("\n【任务】{}", task));

        messages.push(PromptMessage {
            role: "user".to_string(),
            content: user_prompt,
            reasoning_content: None,
            ..Default::default()
        });

        Prompt { messages }
    }
}

/// 魂长驻进程
pub struct SoulProcess {
    /// 魂名称
    pub soul_name: String,
    /// 进程状态
    pub state: RwLock<SoulProcessState>,
    /// 进程上下文
    pub context: RwLock<SoulContext>,
    /// 事件发送通道（内部）
    tx: mpsc::Sender<SoulProcessEvent>,
    /// 干预发送通道（外部通过 SoulProcessManager 注入干预指令）
    pub intervention_tx: mpsc::Sender<Intervention>,
}

impl SoulProcess {
    /// 创建新的魂进程（不启动）
    ///
    /// 返回进程实例、事件接收通道和干预接收通道。
    /// 调用者需要将 `event_rx` 和 `intervention_rx` 传入 `run()` 启动事件循环。
    pub fn new(
        profile: SoulProfile,
    ) -> (
        Self,
        mpsc::Receiver<SoulProcessEvent>,
        mpsc::Receiver<Intervention>,
    ) {
        let (tx, rx) = mpsc::channel(32);
        let (intervention_tx, intervention_rx) = mpsc::channel(32);
        let process = SoulProcess {
            soul_name: profile.name.clone(),
            state: RwLock::new(SoulProcessState::Stopped),
            context: RwLock::new(SoulContext::new(profile)),
            tx,
            intervention_tx,
        };
        (process, rx, intervention_rx)
    }

    /// 发送任务给魂进程
    pub async fn send_task(&self, task: String, presets: Option<UserPresets>) -> Result<()> {
        self.tx
            .send(SoulProcessEvent::Task { task, presets })
            .await
            .map_err(|e| foundation::FoundationError::InvalidState(format!("Failed to send task: {}", e)))?;
        Ok(())
    }

    /// 休眠魂进程
    pub async fn sleep(&self) -> Result<()> {
        self.tx
            .send(SoulProcessEvent::Sleep)
            .await
            .map_err(|e| foundation::FoundationError::InvalidState(format!("Failed to send sleep: {}", e)))?;
        Ok(())
    }

    /// 唤醒魂进程
    pub async fn wake(&self) -> Result<()> {
        self.tx
            .send(SoulProcessEvent::Wake)
            .await
            .map_err(|e| foundation::FoundationError::InvalidState(format!("Failed to send wake: {}", e)))?;
        Ok(())
    }

    /// 停止魂进程
    pub async fn stop(&self) -> Result<()> {
        self.tx
            .send(SoulProcessEvent::Stop)
            .await
            .map_err(|e| foundation::FoundationError::InvalidState(format!("Failed to send stop: {}", e)))?;
        Ok(())
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> SoulProcessState {
        self.state.read().await.clone()
    }

    /// 运行魂进程（在独立任务中） - 接受 Arc<Self> 以支持多所有权
    pub async fn run(
        self: Arc<Self>,
        mut rx: mpsc::Receiver<SoulProcessEvent>,
        mut intervention_rx: mpsc::Receiver<Intervention>,
        gateway: Arc<GatewayRegistry>,
        output_tx: mpsc::Sender<SoulProcessOutput>,
    ) {
        let soul_name = self.soul_name.clone();
        tracing::info!("Soul process started: {}", soul_name);

        loop {
            let state = self.get_state().await;

            tokio::select! {
                maybe_event = rx.recv() => {
                    match maybe_event {
                        Some(event) => match event {
                            SoulProcessEvent::Task { task, presets } => {
                                if state == SoulProcessState::Sleeping {
                                    tracing::info!("Waking up soul {} to process task", soul_name);
                                    *self.state.write().await = SoulProcessState::Active;
                                }

                                self.process_task(
                                    task,
                                    presets.unwrap_or(UserPresets {
                                        judgment: None,
                                        worry: None,
                                        unknown: None,
                                    }),
                                    &gateway,
                                    &output_tx,
                                    &mut intervention_rx,
                                ).await;
                            }
                            SoulProcessEvent::Sleep => {
                                tracing::info!("Putting soul {} to sleep", soul_name);
                                *self.state.write().await = SoulProcessState::Sleeping;
                            }
                            SoulProcessEvent::Wake => {
                                tracing::info!("Waking up soul {}", soul_name);
                                *self.state.write().await = SoulProcessState::Active;
                            }
                            SoulProcessEvent::Stop => {
                                tracing::info!("Stopping soul process: {}", soul_name);
                                *self.state.write().await = SoulProcessState::Stopped;
                                break;
                            }
                        }
                        None => {
                            tracing::warn!("Soul process event channel closed for {}", soul_name);
                            *self.state.write().await = SoulProcessState::Stopped;
                            break;
                        }
                    }
                }
            }
        }
    }

    /// 处理任务 — 带干预感知的流式推理
    ///
    /// 使用 tokio::select! 在流式接收 LLM token 的同时，
    /// 监听干预通道。一旦收到干预指令，立即中断当前推理，
    /// 将干预消息注入上下文后重新启动推理。
    async fn process_task(
        &self,
        task: String,
        presets: UserPresets,
        gateway: &GatewayRegistry,
        output_tx: &mpsc::Sender<SoulProcessOutput>,
        intervention_rx: &mut mpsc::Receiver<Intervention>,
    ) {
        let (profile, base_prompt) = {
            let ctx = self.context.read().await;
            (ctx.profile.clone(), ctx.build_prompt(&task, &presets))
        };

        let provider = if profile.model.contains("deepseek") {
            Provider::DeepSeek
        } else if profile.model.contains("gpt")
            || profile.model.contains("o1")
            || profile.model.contains("o3")
        {
            Provider::OpenAI
        } else {
            Provider::Claude
        };

        let base_config = foundation::CallConfig {
            temperature: 0.7,
            max_tokens: 16384,
            stream: true,
            model: if profile.model.is_empty() {
                None
            } else {
                Some(profile.model.clone())
            },
            tools: None,
            tool_choice: None,
            reasoning_effort: Some(foundation::ReasoningEffort::Think),
            structured_output: None,
            thinking_enabled: None,
        };

        // 干预重试循环：每次被干预后，用注入干预消息的 prompt 重新推理
        let mut current_prompt = base_prompt;
        loop {
            let req = LLMRequest {
                provider,
                prompt: current_prompt.clone(),
                config: base_config.clone(),
            };

            let interrupted = self
                .stream_with_intervention(req, &task, gateway, output_tx, intervention_rx)
                .await;

            match interrupted {
                StreamResult::Completed => break,
                StreamResult::Interrupted {
                    partial_content,
                    intervention,
                } => {
                    tracing::info!(
                        "Soul {} interrupted by {:?}, restarting inference",
                        self.soul_name,
                        std::mem::discriminant(&intervention)
                    );

                    // 将干预消息注入当前 prompt
                    current_prompt
                        .messages
                        .push(intervention.to_prompt_message());

                    // 同时注入部分已完成的内容，让魂在已有基础上调整
                    if !partial_content.is_empty() {
                        current_prompt.messages.push(PromptMessage {
                            role: "assistant".to_string(),
                            content: format!(
                                "（以下是你被中断前已经生成的部分内容，请在此基础上修正）\n{}",
                                partial_content
                            ),
                            reasoning_content: None,
                            ..Default::default()
                        });
                    }
                }
                StreamResult::Error(err_msg) => {
                    let _ = output_tx.send(SoulProcessOutput::Error(err_msg)).await;
                    break;
                }
            }
        }
    }

    /// 流式推理 + 干预感知的 tokio::select! 竞态
    ///
    /// 同时等待两个未来：
    /// - chunk_rx.recv(): LLM token 流
    /// - intervention_rx.recv(): 干预指令
    ///
    /// 任一分支先就绪即执行，另一分支被取消。
    async fn stream_with_intervention(
        &self,
        req: LLMRequest,
        task: &str,
        gateway: &GatewayRegistry,
        output_tx: &mpsc::Sender<SoulProcessOutput>,
        intervention_rx: &mut mpsc::Receiver<Intervention>,
    ) -> StreamResult {
        match gateway.call(&req) {
            Ok(mut chunk_rx) => {
                let mut full_content = String::new();
                let mut full_reasoning = String::new();
                let mut final_usage = UsageStats::default();

                loop {
                    tokio::select! {
                        // 分支 A: 收到 LLM token chunk
                        chunk_opt = chunk_rx.recv() => {
                            match chunk_opt {
                                Some(Ok(chunk)) => {
                                    // 收集回答内容（仅当非 reasoning 时输出）
                                    if !chunk.content.is_empty() && chunk.reasoning_content.is_none() {
                                        full_content.push_str(&chunk.content);
                                        let _ = output_tx
                                            .send(SoulProcessOutput::Chunk(chunk.content))
                                            .await;
                                    }
                                    // 收集思维链
                                    if let Some(reasoning) = &chunk.reasoning_content {
                                        if !reasoning.is_empty() {
                                            full_reasoning.push_str(reasoning);
                                        }
                                    }
                                    // 更新 usage
                                    if let Some(usage) = chunk.usage {
                                        final_usage = usage;
                                    }
                                    // 正常结束
                                    if let Some(reason) = &chunk.finish_reason {
                                        if reason == "stop" || reason == "length" {
                                            {
                                                let mut ctx = self.context.write().await;
                                                ctx.add_message(
                                                    "user".to_string(),
                                                    task.to_string(),
                                                    None,
                                                );
                                                ctx.add_message(
                                                    "assistant".to_string(),
                                                    full_content.clone(),
                                                    Some(full_reasoning.clone()),
                                                );
                                            }
                                            let _ = output_tx
                                                .send(SoulProcessOutput::Done {
                                                    content: full_content,
                                                    usage: final_usage,
                                                })
                                                .await;
                                            return StreamResult::Completed;
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    let err_msg = format!("Error calling LLM: {}", e);
                                    tracing::error!("{}", err_msg);
                                    return StreamResult::Error(err_msg);
                                }
                                None => {
                                    // 流意外关闭
                                    return StreamResult::Completed;
                                }
                            }
                        }
                        // 分支 B: 收到干预指令（推理与干预竞态）
                        intervention_opt = intervention_rx.recv() => {
                            match intervention_opt {
                                Some(intervention) => {
                                    let _ = output_tx
                                        .send(SoulProcessOutput::Interrupted {
                                            partial_content: full_content.clone(),
                                            intervention: intervention.clone(),
                                        })
                                        .await;
                                    return StreamResult::Interrupted {
                                        partial_content: full_content,
                                        intervention,
                                    };
                                }
                                None => {
                                    // 干预通道已关闭，继续等待流完成
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to start LLM call: {}", e);
                tracing::error!("{}", err_msg);
                StreamResult::Error(err_msg)
            }
        }
    }
}

/// 流式推理的返回结果
enum StreamResult {
    /// 推理正常完成
    Completed,
    /// 被干预中断（携带部分内容）
    Interrupted {
        partial_content: String,
        intervention: Intervention,
    },
    /// 发生错误
    Error(String),
}

/// 魂进程管理器
pub struct SoulProcessManager {
    /// 活跃进程
    processes: RwLock<std::collections::HashMap<String, Arc<SoulProcess>>>,
    /// 进程输出通道
    output_channels: RwLock<std::collections::HashMap<String, mpsc::Sender<SoulProcessOutput>>>,
    /// 网关
    gateway: Arc<GatewayRegistry>,
}

impl SoulProcessManager {
    /// 创建新的管理器
    pub fn new(gateway: Arc<GatewayRegistry>) -> Self {
        SoulProcessManager {
            processes: RwLock::new(std::collections::HashMap::new()),
            output_channels: RwLock::new(std::collections::HashMap::new()),
            gateway,
        }
    }

    /// 启动魂进程
    pub async fn start_process(
        &self,
        profile: SoulProfile,
    ) -> Result<(Arc<SoulProcess>, mpsc::Receiver<SoulProcessOutput>)> {
        let soul_name = profile.name.clone();
        
        // 检查是否已存在
        {
            let processes = self.processes.read().await;
            if processes.contains_key(&soul_name) {
                return Err(foundation::FoundationError::Validation(
                    format!("Soul process already running: {}", soul_name)
                ));
            }
        }

        let (process, rx, intervention_rx) = SoulProcess::new(profile);
        let (output_tx, output_rx) = mpsc::channel(256);
        let arc_process = Arc::new(process);

        // 保存进程和输出通道
        {
            let mut processes = self.processes.write().await;
            processes.insert(soul_name.clone(), arc_process.clone());

            let mut channels = self.output_channels.write().await;
            channels.insert(soul_name.clone(), output_tx.clone());
        }

        // 设置状态为活跃
        *arc_process.state.write().await = SoulProcessState::Active;

        // 启动任务 — 传入干预通道使推理与干预可竞态
        let gateway = self.gateway.clone();
        let process_clone = arc_process.clone();
        tokio::spawn(async move {
            process_clone.run(rx, intervention_rx, gateway, output_tx).await;
        });

        Ok((arc_process, output_rx))
    }

    /// 获取魂进程
    pub async fn get_process(&self, soul_name: &str) -> Option<Arc<SoulProcess>> {
        self.processes.read().await.get(soul_name).cloned()
    }

    /// 停止魂进程
    pub async fn stop_process(&self, soul_name: &str) -> Result<()> {
        if let Some(process) = self.get_process(soul_name).await {
            process.stop().await?;
            // 清理
            let mut processes = self.processes.write().await;
            processes.remove(soul_name);
            let mut channels = self.output_channels.write().await;
            channels.remove(soul_name);
        }
        Ok(())
    }

    /// 获取所有活跃进程
    pub async fn list_processes(&self) -> Vec<(String, SoulProcessState)> {
        let processes = self.processes.read().await;
        let mut result = Vec::new();
        for (name, process) in processes.iter() {
            result.push((name.clone(), process.get_state().await));
        }
        result
    }

    /// 向指定魂进程注入干预指令
    ///
    /// 若魂正在推理中，干预将通过 tokio::select! 竞态
    /// 被捕获并注入上下文，触发重新推理。
    pub async fn send_intervention(
        &self,
        soul_name: &str,
        intervention: Intervention,
    ) -> Result<()> {
        let process = self
            .get_process(soul_name)
            .await
            .ok_or_else(|| foundation::FoundationError::SoulNotFound(soul_name.to_string()))?;
        process
            .intervention_tx
            .send(intervention)
            .await
            .map_err(|e| foundation::FoundationError::InvalidState(format!(
                "Failed to send intervention to {}: {}",
                soul_name, e
            )))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_soul_context() {
        let profile = SoulProfile {
            name: "TestSoul".to_string(),
            ismism_code: "0-0-0-0".to_string(),
            field: "Test".to_string(),
            ontology: "".to_string(),
            epistemology: "".to_string(),
            teleology: "".to_string(),
            domains: vec![],
            exclude_scenarios: vec![],
            summon_count: 0,
            effectiveness: foundation::EffectivenessStats::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
            summon_prompt: "You are a test soul".to_string(),
            practice_observations: vec![],
            title: "".to_string(),
            description: "".to_string(),
            voice: "".to_string(),
            mind: "".to_string(),
            self_declare: "".to_string(),
            skills_expertise: vec![],
            model: "".to_string(),
            tools: "".to_string(),
            trigger_keywords: vec![],
            compat: vec![],
            incompat: vec![],
        };

        let mut ctx = SoulContext::new(profile);
        ctx.add_message("user".to_string(), "Hello".to_string(), None);
        ctx.add_message("assistant".to_string(), "Hi there".to_string(), None);

        assert_eq!(ctx.history.len(), 2);
    }
}