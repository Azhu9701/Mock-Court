export interface Preset {
  name: string;
  label: string;
  description: string;
  souls: string[];
  constraint: string;
  scenarios: string[];
}

export const PRESETS: Preset[] = [
  { name: "tech-critique", label: "技术批判", description: "分析技术对社会的结构化影响", souls: ["列宁", "费曼", "未明子"], constraint: "聚焦技术-社会关系，避免纯技术实现讨论", scenarios: ["AI影响", "自动化", "平台经济", "技术治理"] },
  { name: "strategy", label: "战略决策", description: "多维度战略分析", souls: ["毛泽东", "邓小平", "稻盛和夫"], constraint: "聚焦可执行性，输出需含行动建议", scenarios: ["商业战略", "组织决策", "资源配置"] },
  { name: "ideology", label: "意识形态", description: "批判性审视观念背后的权力结构", souls: ["未明子", "葛兰西", "法农"], constraint: "聚焦意识形态运作机制", scenarios: ["文化批判", "媒体分析", "教育批判"] },
  { name: "education", label: "教育分析", description: "教学方法与学习机制", souls: ["费曼", "Karpathy", "孔子"], constraint: "聚焦教学方法和学习效果", scenarios: ["教育设计", "学习方法", "培训方案"] },
  { name: "organization", label: "组织建设", description: "群体行动与组织形式", souls: ["列宁", "毛泽东", "稻盛和夫"], constraint: "聚焦组织效能和可持续性", scenarios: ["团队建设", "组织架构", "流程设计"] },
  { name: "gender-race", label: "性别种族", description: "结构性压迫分析", souls: ["波伏娃", "法农", "鲁迅"], constraint: "聚焦结构性因素", scenarios: ["性别分析", "种族批判", "交叉性研究"] },
  { name: "existential", label: "存在意义", description: "人生意义与存在追问", souls: ["尼采", "庄子", "波伏娃"], constraint: "聚焦个体存在体验", scenarios: ["人生选择", "意义危机", "自由与责任"] },
  { name: "epistemology", label: "认识论", description: "知识的本质与获取方式", souls: ["费曼", "胡塞尔", "马克思"], constraint: "聚焦认识方法而非具体知识", scenarios: ["研究方法", "认知偏见", "学科交叉"] },
  { name: "labor", label: "劳动分析", description: "劳动与生产关系分析", souls: ["马克思", "毛泽东", "祝鹤槐"], constraint: "聚焦生产关系和劳动过程", scenarios: ["劳资关系", "工作设计", "自动化替代"] },
  { name: "spirit", label: "精神修养", description: "内在成长与文化修养", souls: ["孔子", "庄子", "稻盛和夫"], constraint: "聚焦个人修养和内在成长", scenarios: ["道德困境", "人生规划", "自我提升"] },
];
