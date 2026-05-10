export const DEEPSEEK_MODELS = [
  { value: "", label: "使用默认配置" },
  { value: "deepseek-v4-pro", label: "DeepSeek V4 Pro (deepseek-v4-pro) ★ 推荐" },
  { value: "deepseek-v4-flash", label: "DeepSeek V4 Flash (deepseek-v4-flash) - 快速响应" },
  { value: "deepseek-chat", label: "DeepSeek Chat (deepseek-chat) ⚠️ 弃用" },
  { value: "deepseek-coder", label: "DeepSeek Coder (deepseek-coder)" },
] as const;

export const DEEPSEEK_MODELS_NO_DEFAULT = DEEPSEEK_MODELS.filter(m => m.value !== "") as unknown as { value: string; label: string }[];

export const REASONING_OPTIONS = [
  { value: "", label: "使用默认配置" },
  { value: "non-think", label: "Non-Think（无推理，快速响应）" },
  { value: "think", label: "Think（标准推理）" },
  { value: "think-high", label: "Think High（深度推理）" },
  { value: "think-max", label: "Think Max（最大推理）" },
] as const;
