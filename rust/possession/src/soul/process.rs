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
    /// 错误
    Error(String),
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
    /// 事件发送通道
    tx: mpsc::Sender<SoulProcessEvent>,
}

impl SoulProcess {
    /// 创建新的魂进程（不启动）
    pub fn new(profile: SoulProfile) -> (Self, mpsc::Receiver<SoulProcessEvent>) {
        let (tx, rx) = mpsc::channel(32);
        let process = SoulProcess {
            soul_name: profile.name.clone(),
            state: RwLock::new(SoulProcessState::Stopped),
            context: RwLock::new(SoulContext::new(profile)),
            tx,
        };
        (process, rx)
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
        gateway: Arc<GatewayRegistry>,
        output_tx: mpsc::Sender<SoulProcessOutput>,
    ) {
        let soul_name = self.soul_name.clone();
        tracing::info!("Soul process started: {}", soul_name);

        loop {
            let state = self.get_state().await;
            
            tokio::select! {
                Some(event) = rx.recv() => {
                    match event {
                        SoulProcessEvent::Task { task, presets } => {
                            if state == SoulProcessState::Sleeping {
                                tracing::info!("Waking up soul {} to process task", soul_name);
                                *self.state.write().await = SoulProcessState::Active;
                            }
                            
                            self.process_task(task, presets.unwrap_or(UserPresets {
                                judgment: None,
                                worry: None,
                                unknown: None,
                            }), &gateway, &output_tx).await;
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
                }
            }
        }
    }

    async fn process_task(
        &self,
        task: String,
        presets: UserPresets,
        gateway: &GatewayRegistry,
        output_tx: &mpsc::Sender<SoulProcessOutput>,
    ) {
        let (profile, prompt) = {
            let ctx = self.context.read().await;
            (ctx.profile.clone(), ctx.build_prompt(&task, &presets))
        };

        let provider = if profile.model.contains("deepseek") {
            Provider::DeepSeek
        } else if profile.model.contains("gpt") || profile.model.contains("o1") || profile.model.contains("o3") {
            Provider::OpenAI
        } else {
            Provider::Claude
        };

        let config = foundation::CallConfig {
            temperature: 0.7,
            max_tokens: 8192,
            stream: true,
            model: if profile.model.is_empty() { None } else { Some(profile.model.clone()) },
            tools: None,
            tool_choice: None,
            reasoning_effort: Some(foundation::ReasoningEffort::Think),
            structured_output: None,
            thinking_enabled: None,
        };

        let req = LLMRequest {
            provider,
            prompt,
            config,
        };

        match gateway.call(&req) {
            Ok(mut rx) => {
                let mut full_content = String::new();
                let mut full_reasoning = String::new();
                let mut final_usage = UsageStats::default();

                while let Some(chunk_result) = rx.recv().await {
                    match chunk_result {
                        Ok(chunk) => {
                            // 收集最终回答内容
                            if !chunk.content.is_empty() && chunk.reasoning_content.is_none() {
                                full_content.push_str(&chunk.content);
                                let _ = output_tx.send(SoulProcessOutput::Chunk(chunk.content)).await;
                            }
                            // 收集思维链内容（用于多轮对话）
                            if let Some(reasoning) = &chunk.reasoning_content {
                                if !reasoning.is_empty() {
                                    full_reasoning.push_str(reasoning);
                                }
                            }
                            if let Some(usage) = chunk.usage {
                                final_usage = usage;
                            }
                            if let Some(reason) = &chunk.finish_reason {
                                if reason == "stop" || reason == "length" {
                                    // 保存到历史（带思维链用于多轮对话）
                                    {
                                        let mut ctx = self.context.write().await;
                                        ctx.add_message("user".to_string(), task.clone(), None);
                                        ctx.add_message("assistant".to_string(), full_content.clone(), Some(full_reasoning.clone()));
                                    }

                                    let _ = output_tx.send(SoulProcessOutput::Done {
                                        content: full_content,
                                        usage: final_usage,
                                    }).await;
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Error calling LLM: {}", e);
                            tracing::error!("{}", err_msg);
                            let _ = output_tx.send(SoulProcessOutput::Error(err_msg)).await;
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to start LLM call: {}", e);
                tracing::error!("{}", err_msg);
                let _ = output_tx.send(SoulProcessOutput::Error(err_msg)).await;
            }
        }
    }
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

        let (process, rx) = SoulProcess::new(profile);
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
        
        // 启动任务
        let gateway = self.gateway.clone();
        let process_clone = arc_process.clone();
        tokio::spawn(async move {
            process_clone.run(rx, gateway, output_tx).await;
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