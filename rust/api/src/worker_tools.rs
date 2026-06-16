//! Worker Rights Tools — 劳动者权益专用工具
//!
//! 提供补偿金计算、证据清单生成、劳动法知识检索三个工具，
//! 供工友智囊团中的 AI Soul 在分析劳动者处境时调用。

use async_trait::async_trait;
use possession::tools::ToolHandler;

// ═══════════════════════════════════════════════════════════════════════════
// CalculateSeveranceTool — 离职补偿金计算器
// ═══════════════════════════════════════════════════════════════════════════

pub struct CalculateSeveranceTool;

#[async_trait]
impl ToolHandler for CalculateSeveranceTool {
    fn name(&self) -> &str {
        "calculate_severance"
    }

    fn description(&self) -> &str {
        "计算劳动者离职时应得的补偿金/赔偿金。\
        根据工龄、月工资、离职原因，计算 N（经济补偿金）、N+1（加代通知金）、\
        2N（违法解除赔偿金）的具体金额。同时估算未休年假折算和加班费。\
        适用于被辞退、裁员、协商解除等场景。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "years_of_service": {
                    "type": "number",
                    "description": "在公司的连续工作年限（年），不满半年按0.5年计，满半年不满1年按1年计"
                },
                "monthly_salary": {
                    "type": "number",
                    "description": "离职前12个月的平均月工资（含奖金、津贴、加班费等）。如果超过当地社平工资3倍，按3倍封顶"
                },
                "termination_reason": {
                    "type": "string",
                    "enum": ["illegal", "no_fault", "fault", "layoff", "mutual_agreement"],
                    "description": "离职原因：illegal=违法解除(公司无合法理由辞退), no_fault=无过错解除(裁员/合同到期不续/医疗期满等), fault=过错解除(劳动者严重违纪等), layoff=经济性裁员, mutual_agreement=协商一致解除"
                },
                "local_avg_salary_3x": {
                    "type": "number",
                    "description": "当地上年度职工月平均工资的3倍（封顶线）。如不清楚可填0，系统将提示"
                },
                "unused_annual_leave_days": {
                    "type": "number",
                    "description": "当年度已应计但未休的年假天数"
                },
                "unpaid_overtime_hours": {
                    "type": "number",
                    "description": "尚未结算的加班小时数（如有）"
                },
                "monthly_salary_is_before_tax": {
                    "type": "boolean",
                    "description": "月工资是税前(true)还是税后(false)"
                }
            },
            "required": ["years_of_service", "monthly_salary", "termination_reason"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| foundation::FoundationError::Validation(format!("参数解析失败: {e}")))?;

        let years: f64 = args["years_of_service"]
            .as_f64()
            .ok_or_else(|| foundation::FoundationError::Validation("缺少 years_of_service 参数".into()))?;

        let monthly: f64 = args["monthly_salary"]
            .as_f64()
            .ok_or_else(|| foundation::FoundationError::Validation("缺少 monthly_salary 参数".into()))?;

        let reason = args["termination_reason"]
            .as_str()
            .ok_or_else(|| foundation::FoundationError::Validation("缺少 termination_reason 参数".into()))?;

        let cap_3x = args.get("local_avg_salary_3x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let leave_days: f64 = args.get("unused_annual_leave_days").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let overtime_hours: f64 = args.get("unpaid_overtime_hours").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let is_pretax = args.get("monthly_salary_is_before_tax").and_then(|v| v.as_bool()).unwrap_or(true);

        // 计算补偿年限（N）：不满半年=0.5，满半年不满1年=1
        let n_years = (years * 2.0).ceil() / 2.0;

        // 封顶工资
        let effective_monthly = if cap_3x > 0.0 && monthly > cap_3x {
            cap_3x
        } else {
            monthly
        };

        let is_capped = cap_3x > 0.0 && monthly > cap_3x;

        let daily_wage = effective_monthly / 21.75; // 月计薪天数

        let mut result = String::new();
        result.push_str("## 离职补偿金计算结果\n\n");

        result.push_str(&format!(
            "- **工作年限**：{} 年 → 计补偿年限 N = **{}**\n",
            years, n_years
        ));
        result.push_str(&format!(
            "- **月工资基数**：¥{:.2}/月{}\n",
            effective_monthly,
            if is_capped { format!("（原工资 ¥{:.2}，已按3倍社平工资 ¥{:.2} 封顶）", monthly, cap_3x) } else { String::new() }
        ));
        if !is_pretax {
            result.push_str("  ⚠️ 输入的月工资为税后，实际计算应以税前工资为准\n");
        }
        result.push('\n');

        // ── 核心补偿计算 ──
        result.push_str("### 补偿金/赔偿金\n\n");

        match reason {
            "illegal" => {
                // 违法解除 = 2N
                let severance_2n = effective_monthly * n_years * 2.0;
                result.push_str("**离职原因：违法解除** — 公司无合法理由单方辞退\n\n");
                result.push_str(&format!("| 项目 | 公式 | 金额 |\n"));
                result.push_str(&format!("|------|------|------|\n"));
                result.push_str(&format!(
                    "| **违法解除赔偿金 (2N)** | ¥{:.2} × {} × 2 | **¥{:.2}** |\n",
                    effective_monthly, n_years, severance_2n
                ));
                result.push_str(&format!(
                    "\n> 💡 **你可以主张 2N = ¥{:.2}**。这是公司违法辞退你应支付的赔偿金。\n",
                    severance_2n
                ));
                result.push_str("> 如果公司以'严重违纪'等理由辞退但拿不出证据，就属于违法解除。\n");
            }
            "no_fault" => {
                // 无过错解除 = N+1
                let severance_n1 = effective_monthly * n_years + effective_monthly;
                result.push_str("**离职原因：无过错解除** — 裁员/合同到期不续/医疗期满等\n\n");
                result.push_str(&format!("| 项目 | 公式 | 金额 |\n"));
                result.push_str(&format!("|------|------|------|\n"));
                result.push_str(&format!(
                    "| 经济补偿金 (N) | ¥{:.2} × {} | ¥{:.2} |\n",
                    effective_monthly, n_years, effective_monthly * n_years
                ));
                result.push_str(&format!(
                    "| 代通知金 (+1) | ¥{:.2} × 1 | ¥{:.2} |\n",
                    effective_monthly, effective_monthly
                ));
                result.push_str(&format!(
                    "| **合计 (N+1)** | | **¥{:.2}** |\n",
                    severance_n1
                ));
                result.push_str(&format!(
                    "\n> 💡 **你可以主张 N+1 = ¥{:.2}**。如果公司提前30天书面通知，则只需付N（¥{:.2}）。\n",
                    severance_n1, effective_monthly * n_years
                ));
            }
            "fault" => {
                result.push_str("**离职原因：过错解除** — 劳动者严重违纪等\n\n");
                result.push_str("> ⚠️ 如果公司确实能证明你存在严重违纪行为，可能无需支付补偿金。\n");
                result.push_str("> 但公司必须承担举证责任——口头说'违纪'不算。请确认公司是否有书面证据。\n");
                result.push_str(&format!(
                    "> 如果公司证据不足，你可以主张违法解除赔偿金 **2N = ¥{:.2}**\n",
                    effective_monthly * n_years * 2.0
                ));
            }
            "layoff" => {
                // 经济性裁员 = N
                let severance_n = effective_monthly * n_years;
                result.push_str("**离职原因：经济性裁员**\n\n");
                result.push_str(&format!("| 项目 | 公式 | 金额 |\n"));
                result.push_str(&format!("|------|------|------|\n"));
                result.push_str(&format!(
                    "| **经济补偿金 (N)** | ¥{:.2} × {} | **¥{:.2}** |\n",
                    effective_monthly, n_years, severance_n
                ));
                result.push_str(&format!(
                    "\n> 💡 **你可以主张 N = ¥{:.2}**。经济性裁员需满足法定程序（提前30天说明情况、向劳动部门报告等），如公司未履行程序，可能构成违法解除。\n",
                    severance_n
                ));
            }
            "mutual_agreement" => {
                let severance_n = effective_monthly * n_years;
                result.push_str("**离职原因：协商一致解除**\n\n");
                result.push_str(&format!(
                    "> 💡 协商解除的补偿金额由双方协商确定。参照标准：**N = ¥{:.2}**（¥{:.2} × {}年）。\n",
                    severance_n, effective_monthly, n_years
                ));
                result.push_str("> 协商时可以 N 为基础进行谈判。如果公司主动提出协商解除，可以争取 N+1 或更高。\n");
            }
            _ => {
                return Err(foundation::FoundationError::Validation(
                    format!("不支持的离职原因类型: {}", reason)
                ));
            }
        }

        // ── 未休年假折算 ──
        if leave_days > 0.0 {
            let leave_pay = daily_wage * leave_days * 3.0; // 未休年假按300%支付
            result.push_str("\n### 未休年假折算\n\n");
            result.push_str(&format!(
                "- 未休年假天数：{} 天\n- 日工资：¥{:.2}\n- 折算金额（300%）：**¥{:.2}**\n",
                leave_days, daily_wage, leave_pay
            ));
            result.push_str("> 根据规定，公司未安排年休假应按日工资的300%支付。\n");
        }

        // ── 加班费估算 ──
        if overtime_hours > 0.0 {
            let hourly = effective_monthly / 21.75 / 8.0;
            let ot_pay = hourly * overtime_hours * 1.5; // 按1.5倍估算
            result.push_str("\n### 未结加班费估算\n\n");
            result.push_str(&format!(
                "- 未结加班小时数：{} 小时\n- 小时工资：¥{:.2}\n- 估算加班费（1.5倍）：**¥{:.2}**（实际费率以加班类型为准：工作日1.5倍/休息日2倍/法定假日3倍）\n",
                overtime_hours, hourly, ot_pay
            ));
        }

        // ── 总计 ──
        result.push_str("\n---\n");
        result.push_str("> ⚠️ **重要提示**：\n");
        result.push_str("> 1. 以上计算基于通用劳动法原则，具体金额可能因所在地区的法律规定和司法实践而异\n");
        result.push_str("> 2. 封顶线各地不同——如果你所在地区的社平工资3倍封顶线不同，请自行调整\n");
        result.push_str("> 3. 补偿金计算基数包含奖金、津贴、加班费等——公司可能只按基本工资计算，这是不对的\n");
        result.push_str("> 4. 建议收集工资条、银行流水、劳动合同等证据后咨询当地劳动仲裁或律师\n");

        Ok(result)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EvidenceChecklistTool — 证据清单生成器
// ═══════════════════════════════════════════════════════════════════════════

pub struct EvidenceChecklistTool;

#[async_trait]
impl ToolHandler for EvidenceChecklistTool {
    fn name(&self) -> &str {
        "generate_evidence_checklist"
    }

    fn description(&self) -> &str {
        "根据劳动者面临的案件类型，生成针对性的证据收集清单。\
        涵盖辞退、工资纠纷、工伤、歧视等场景，逐项说明需要收集什么证据、\
        从哪里获取、为什么重要、以及收集时的注意事项。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "case_type": {
                    "type": "string",
                    "enum": ["dismissal", "wage_dispute", "injury", "discrimination", "contract_dispute", "general"],
                    "description": "案件类型：dismissal=辞退/裁员, wage_dispute=工资/加班费纠纷, injury=工伤/职业病, discrimination=职场歧视/骚扰, contract_dispute=合同纠纷, general=通用"
                },
                "specific_details": {
                    "type": "string",
                    "description": "案件的具体情况简述（如'公司以不能胜任工作为由辞退，但未提供培训或调岗'），用于生成更针对性的建议"
                }
            },
            "required": ["case_type"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| foundation::FoundationError::Validation(format!("参数解析失败: {e}")))?;

        let case_type = args["case_type"]
            .as_str()
            .ok_or_else(|| foundation::FoundationError::Validation("缺少 case_type 参数".into()))?;

        let details = args.get("specific_details")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut result = String::new();
        result.push_str("## 证据收集清单\n\n");

        if !details.is_empty() {
            result.push_str(&format!("**案件情况**：{}\n\n", details));
        }

        result.push_str("以下是你应该收集的证据。**越早收集越好**——一旦离职，公司邮箱、内部系统、打卡记录可能无法访问。\n\n");

        match case_type {
            "dismissal" => dismissal_checklist(&mut result),
            "wage_dispute" => wage_checklist(&mut result),
            "injury" => injury_checklist(&mut result),
            "discrimination" => discrimination_checklist(&mut result),
            "contract_dispute" => contract_checklist(&mut result),
            "general" => general_checklist(&mut result),
            _ => general_checklist(&mut result),
        }

        result.push_str("\n---\n");
        result.push_str("### 收集证据的通用原则\n\n");
        result.push_str("1. **先保存再离职**：离职前务必把公司邮箱、聊天记录、打卡记录等导出到个人设备\n");
        result.push_str("2. **原件优于复印件**：纸质文件拍照或扫描，电子文件截图并保留原始文件\n");
        result.push_str("3. **时间线记录**：按时间顺序整理事件经过，标注每次沟通的日期、人员、内容\n");
        result.push_str("4. **录音注意事项**：与HR或领导沟通时录音——在很多地区单方录音可以作为证据\n");
        result.push_str("5. **不要只依赖公司**：社保记录去社保局查，工资流水去银行打——不要只看公司提供的数据\n");
        result.push_str("6. **备份备份再备份**：所有电子证据至少保留2份，分别存储在不同设备或云端\n");

        Ok(result)
    }
}

fn dismissal_checklist(result: &mut String) {
    result.push_str("### 辞退/裁员 — 关键证据\n\n");
    result.push_str("| 序号 | 证据 | 来源 | 为什么重要 |\n");
    result.push_str("|------|------|------|------------|\n");
    result.push_str("| 1 | **解除劳动合同通知书** | HR/公司 | 这是最核心的证据——上面写的辞退理由决定了你能拿N还是2N |\n");
    result.push_str("| 2 | **劳动合同** | 自己保存 | 确认岗位、工资、合同期限——公司辞退理由是否与合同约定冲突 |\n");
    result.push_str("| 3 | **工资条/银行流水** | 公司/银行 | 证明你的实际工资（含奖金津贴）——计算补偿金的基数 |\n");
    result.push_str("| 4 | **社保缴纳记录** | 社保局/App | 证明劳动关系存续期间——如果公司未足额缴纳也是问题 |\n");
    result.push_str("| 5 | **与HR/领导的沟通记录** | 微信/钉钉/邮件 | 辞退前后的沟通——公司可能在口头承诺和书面文件中说法不一致 |\n");
    result.push_str("| 6 | **考勤/打卡记录** | 公司系统 | 证明你在职——如果公司删除了你的账号就无法自行获取 |\n");
    result.push_str("| 7 | **绩效考核记录** | 公司系统/邮件 | 如果公司以'不能胜任'为理由辞退——你的绩效记录是最好的反驳 |\n");
    result.push_str("| 8 | **工作交接记录** | 自己记录 | 你被要求交接了什么、交给了谁——证明你确实在工作 |\n");
    result.push_str("| 9 | **竞业限制协议**（如有） | 自己保存 | 如果签了竞业限制——确认补偿金是否给了、范围是否合理 |\n");
    result.push_str("| 10 | **离职证明** | 公司 | 公司必须出具——不给出具可以投诉 |\n");
}

fn wage_checklist(result: &mut String) {
    result.push_str("### 工资/加班费纠纷 — 关键证据\n\n");
    result.push_str("| 序号 | 证据 | 来源 | 为什么重要 |\n");
    result.push_str("|------|------|------|------------|\n");
    result.push_str("| 1 | **工资条/银行流水** | 公司/银行 | 逐月对比实际发放金额——是否低于合同约定、是否克扣 |\n");
    result.push_str("| 2 | **劳动合同中的工资条款** | 自己保存 | 约定的工资结构和金额——与实际发放对比 |\n");
    result.push_str("| 3 | **加班记录** | 打卡系统/自己记录 | 每天几点到几点——越详细越好。公司不记录加班不代表你没加班 |\n");
    result.push_str("| 4 | **加班审批记录**（如有） | 公司系统/邮件 | 公司要求的加班或你申请获批的加班 |\n");
    result.push_str("| 5 | **加班期间的产出记录** | 邮件/工作系统 | 加班时段发送的邮件、提交的代码/文件——证明你确实在加班工作 |\n");
    result.push_str("| 6 | **与主管关于加班的沟通** | 微信/钉钉 | '今晚把这个做完'——这就是加班指令 |\n");
    result.push_str("| 7 | **社保缴费基数** | 社保局 | 公司是否按实际工资缴纳——不足额缴纳也是工资问题 |\n");
    result.push_str("| 8 | **年终奖/绩效奖金承诺** | 邮件/制度文件 | 如果公司承诺了但没发——书面记录是关键 |\n");
    result.push_str("| 9 | **同事的工资情况**（如可获取） | 同事 | 同工不同酬的证据——但需注意保护同事隐私 |\n");
}

fn injury_checklist(result: &mut String) {
    result.push_str("### 工伤/职业病 — 关键证据\n\n");
    result.push_str("| 序号 | 证据 | 来源 | 为什么重要 |\n");
    result.push_str("|------|------|------|------------|\n");
    result.push_str("| 1 | **事故报告/记录** | 公司/自己 | 工伤发生的时间、地点、经过——第一时间记录，越详细越好 |\n");
    result.push_str("| 2 | **医疗记录** | 医院 | 就诊时明确告诉医生'这是工作中发生的'——让医生写在病历上 |\n");
    result.push_str("| 3 | **诊断证明/病假条** | 医院 | 伤情的专业诊断——工伤认定的核心依据 |\n");
    result.push_str("| 4 | **目击证人证言** | 同事 | 有同事看到事故发生——请他们作证或记录证言 |\n");
    result.push_str("| 5 | **事故现场照片/视频** | 自己/同事 | 事故发生地、设备、环境——第一时间拍摄 |\n");
    result.push_str("| 6 | **工作环境记录** | 自己 | 粉尘、噪音、化学品、高温——长期暴露环境的记录 |\n");
    result.push_str("| 7 | **安全培训记录** | 公司 | 公司是否提供了安全培训——如果没有，是公司的责任 |\n");
    result.push_str("| 8 | **防护用品发放记录** | 公司 | 公司是否提供了防护用品——没提供或提供不合格 |\n");
    result.push_str("| 9 | **职业健康体检报告** | 体检机构 | 入职体检和在职体检——对比健康变化 |\n");
    result.push_str("| 10 | **医疗费用票据** | 医院/药店 | 所有因伤产生的费用——保留每一张票据 |\n");
}

fn discrimination_checklist(result: &mut String) {
    result.push_str("### 职场歧视/骚扰 — 关键证据\n\n");
    result.push_str("| 序号 | 证据 | 来源 | 为什么重要 |\n");
    result.push_str("|------|------|------|------------|\n");
    result.push_str("| 1 | **歧视性言论/行为的记录** | 自己记录 | 谁、什么时间、说了什么/做了什么——逐条记录，越具体越好 |\n");
    result.push_str("| 2 | **聊天记录/邮件** | 微信/钉钉/邮件 | 含有歧视内容的文字信息——截图保存，不要只保留在公司系统里 |\n");
    result.push_str("| 3 | **录音** | 自己录制 | 与HR/领导的对话录音——很多地区单方录音可作为证据 |\n");
    result.push_str("| 4 | **目击证人** | 同事 | 有同事在场听到/看到——但证人可能担心报复，注意保护ta们 |\n");
    result.push_str("| 5 | **同工不同酬的对比数据** | 工资条/公司系统 | 同样岗位同样工作内容不同工资——需要有具体数字对比 |\n");
    result.push_str("| 6 | **招聘/晋升中的歧视证据** | 招聘广告/邮件 | 招聘条件中隐含的歧视（如'限男性''35岁以下'） |\n");
    result.push_str("| 7 | **公司投诉记录** | 公司HR/系统 | 如果你向公司投诉过——保留投诉记录和公司回复 |\n");
    result.push_str("| 8 | **心理/健康影响记录** | 医院/自己 | 歧视导致的心理压力、失眠等——如有就医记录更好 |\n");
    result.push_str("| 9 | **报复行为记录** | 自己 | 投诉后被降职、调岗、排挤——歧视投诉后的报复本身也是违法的 |\n");
}

fn contract_checklist(result: &mut String) {
    result.push_str("### 合同纠纷 — 关键证据\n\n");
    result.push_str("| 序号 | 证据 | 来源 | 为什么重要 |\n");
    result.push_str("|------|------|------|------------|\n");
    result.push_str("| 1 | **劳动合同（全部页面）** | 自己保存 | 核心文件——每一条都可能有用 |\n");
    result.push_str("| 2 | **所有补充协议/变更协议** | 自己保存 | 竞业限制、保密协议、薪资变更等——不是只有主合同才有法律效力 |\n");
    result.push_str("| 3 | **offer letter / 录用通知书** | 自己保存 | 入职前公司承诺的条件——如果与合同不符也是问题 |\n");
    result.push_str("| 4 | **公司规章制度/员工手册** | 公司 | 公司内部的考勤、考核、奖惩制度——辞退时经常引用 |\n");
    result.push_str("| 5 | **转正/续签通知** | 公司 | 试用期是否转正、合同是否续签——如果到期未续签但继续工作，视为无固定期限合同 |\n");
    result.push_str("| 6 | **社保/公积金缴纳记录** | 社保局/公积金中心 | 是否按时足额缴纳——试用期也应当缴纳 |\n");
    result.push_str("| 7 | **岗位/薪资变更记录** | 邮件/工资条 | 调岗、调薪是否有书面记录——口头变更可能有争议 |\n");
}

fn general_checklist(result: &mut String) {
    result.push_str("### 通用证据清单（适用于所有劳动争议）\n\n");
    result.push_str("| 序号 | 证据 | 说明 |\n");
    result.push_str("|------|------|------|\n");
    result.push_str("| 1 | **劳动合同** | 最基础也是最核心的证据 |\n");
    result.push_str("| 2 | **工资银行流水** | 去银行打印最近12-24个月的 |\n");
    result.push_str("| 3 | **社保缴纳记录** | 去社保局或社保App查询 |\n");
    result.push_str("| 4 | **与公司的所有书面沟通** | 微信、钉钉、邮件、书面通知——全部截图/导出 |\n");
    result.push_str("| 5 | **打卡/考勤记录** | 离职前导出——离职后账号可能被注销 |\n");
    result.push_str("| 6 | **工作内容记录** | 邮件、工作产物、项目文件——证明你做了什么 |\n");
    result.push_str("| 7 | **事件时间线** | 自己按时间顺序整理——作为维权时的叙述基础 |\n");
    result.push_str("| 8 | **同事联系方式** | 离职后可能联系不上——提前保存可靠同事的联系方式 |\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// LaborLawSearchTool — 劳动法知识检索
// ═══════════════════════════════════════════════════════════════════════════

pub struct LaborLawSearchTool {
    knowledge_dir: std::path::PathBuf,
}

impl LaborLawSearchTool {
    pub fn new(knowledge_dir: std::path::PathBuf) -> Self {
        // Ensure the directory exists
        let _ = std::fs::create_dir_all(&knowledge_dir);
        LaborLawSearchTool { knowledge_dir }
    }
}

#[async_trait]
impl ToolHandler for LaborLawSearchTool {
    fn name(&self) -> &str {
        "search_labor_law"
    }

    fn description(&self) -> &str {
        "检索劳动法知识库，获取关于劳动者权益的法律原则、国际劳工标准、\
        通用劳动法规则等信息。支持按国家/地区和主题检索。\
        注意：知识库提供通用劳动法原则，具体法条请以当地法律规定为准。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "检索关键词，如'辞退补偿''加班费计算''工伤认定标准''竞业限制'"
                },
                "country": {
                    "type": "string",
                    "description": "国家/地区（可选），如'中国''美国''日本'。如果不指定则返回通用原则"
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| foundation::FoundationError::Validation(format!("参数解析失败: {e}")))?;

        let query = args["query"]
            .as_str()
            .ok_or_else(|| foundation::FoundationError::Validation("缺少 query 参数".into()))?;

        let country = args.get("country").and_then(|v| v.as_str());

        // Search knowledge files
        let mut results = Vec::new();
        self.search_files(query, country, &mut results)?;

        if results.is_empty() {
            // Fallback: provide built-in knowledge for common queries
            return Ok(builtin_labor_knowledge(query));
        }

        let mut output = format!("## 劳动法知识检索: {}\n\n", query);
        if let Some(c) = country {
            output.push_str(&format!("**地区**: {}\n\n", c));
        }
        output.push_str(&format!("找到 {} 条相关结果：\n\n", results.len()));

        for (i, (title, snippet)) in results.iter().enumerate() {
            output.push_str(&format!("{}. **{}**\n", i + 1, title));
            output.push_str(&format!("   {}\n\n", snippet));
        }

        output.push_str("> ⚠️ 以上内容基于通用劳动法原则。请以当地最新法律法规为准。\n");

        Ok(output)
    }
}

impl LaborLawSearchTool {
    fn search_files(
        &self,
        query: &str,
        country: Option<&str>,
        results: &mut Vec<(String, String)>,
    ) -> foundation::Result<()> {
        if !self.knowledge_dir.exists() {
            return Ok(());
        }

        let query_lower = query.to_lowercase();
        let country_lower = country.map(|c| c.to_lowercase());

        let entries = std::fs::read_dir(&self.knowledge_dir)
            .map_err(|e| foundation::FoundationError::Io(e))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }
            let file_name = path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Filter by country if specified
            if let Some(ref c) = country_lower {
                if !file_name.contains(c) && !file_name.contains("universal") {
                    continue;
                }
            }

            // Read and search
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let content_lower = content.to_lowercase();

            if content_lower.contains(&query_lower) {
                // Extract relevant snippet
                let snippet = extract_snippet(&content, &query_lower, 200);
                let title = path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知")
                    .to_string();
                results.push((title, snippet));
            }

            if results.len() >= 5 {
                break;
            }
        }

        Ok(())
    }
}

fn extract_snippet(content: &str, query_lower: &str, max_len: usize) -> String {
    let content_lower = content.to_lowercase();
    if let Some(pos) = content_lower.find(query_lower) {
        let start = pos.saturating_sub(50);
        let end = (pos + query_lower.len() + max_len).min(content.len());
        let snippet = &content[start..end];
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < content.len() { "..." } else { "" };
        format!("{}{}{}", prefix, snippet.trim(), suffix)
    } else {
        content.chars().take(max_len).collect()
    }
}

/// Built-in labor law knowledge for common queries
fn builtin_labor_knowledge(query: &str) -> String {
    let query_lower = query.to_lowercase();

    let mut result = format!("## 劳动法知识: {}\n\n", query);
    result.push_str("> 以下基于国际劳工组织（ILO）核心公约和通用劳动法原则，供参考。\n");
    result.push_str("> 具体法律条文和执行标准请以你所在地区的法律法规为准。\n\n");

    if query_lower.contains("辞退") || query_lower.contains("解雇") || query_lower.contains("补偿") || query_lower.contains("赔偿") {
        result.push_str("### 辞退与补偿通用原则\n\n");
        result.push_str("- **合法辞退**：雇主需有正当理由（如严重违纪、不能胜任且经培训调岗后仍不能胜任、经济性裁员）\n");
        result.push_str("- **经济补偿金（N）**：无过错解除时，按工作年限支付——每满1年支付1个月工资\n");
        result.push_str("- **违法解除赔偿金（2N）**：无合法理由辞退，按经济补偿金的2倍支付\n");
        result.push_str("- **代通知金（+1）**：未提前30天通知即解除，额外支付1个月工资\n");
        result.push_str("- **不得辞退的情形**：孕期/产期/哺乳期、医疗期内、工伤治疗期间、法定退休年龄前5年内（部分地区）\n");
        result.push_str("- **举证责任**：辞退的合法性由雇主承担举证责任——雇主需证明辞退理由成立\n");
        result.push_str("- **ILO第158号公约**：雇主终止雇佣需有正当理由，劳动者有权在合理期限内申诉\n");
    }

    if query_lower.contains("加班") || query_lower.contains("工资") || query_lower.contains("工时") {
        result.push_str("### 工时与加班费通用原则\n\n");
        result.push_str("- **标准工时**：每周不超过40-48小时（各国不同），每日不超过8小时\n");
        result.push_str("- **加班费**：工作日加班1.5倍、休息日加班2倍（或补休）、法定假日加班3倍——此为多数国家的通用标准\n");
        result.push_str("- **强制加班**：雇主不得强迫劳动者加班——加班应以自愿为前提\n");
        result.push_str("- **加班上限**：每月加班通常不超过36小时（各国不同）\n");
        result.push_str("- **最低工资**：雇主支付的工资不得低于法定最低工资标准\n");
        result.push_str("- **工资支付周期**：至少每月支付一次，不得无故拖欠或克扣\n");
        result.push_str("- **ILO第1号和第30号公约**：规定工时限制和休息权\n");
    }

    if query_lower.contains("工伤") || query_lower.contains("职业病") || query_lower.contains("安全") {
        result.push_str("### 工伤与职业安全通用原则\n\n");
        result.push_str("- **工伤认定**：在工作时间和工作场所内因工作原因受到事故伤害，或在上下班途中发生非本人主要责任的交通事故\n");
        result.push_str("- **职业病**：因接触粉尘、放射性物质、有毒有害物质等引起的疾病\n");
        result.push_str("- **雇主义务**：提供安全的工作环境、必要的防护设备、安全培训、定期体检\n");
        result.push_str("- **工伤保险**：雇主须为劳动者缴纳工伤保险——未缴纳的由雇主承担工伤待遇\n");
        result.push_str("- **工伤待遇**：医疗费、停工留薪期工资、伤残津贴、护理费等\n");
        result.push_str("- **ILO第155号公约**：职业安全与健康，雇主有义务确保工作场所安全\n");
    }

    if query_lower.contains("歧视") || query_lower.contains("平等") || query_lower.contains("骚扰") {
        result.push_str("### 反歧视与平等通用原则\n\n");
        result.push_str("- **禁止歧视**：不得因性别、种族、年龄、宗教、残疾、性取向、婚姻状况等因素在招聘、晋升、薪酬等方面区别对待\n");
        result.push_str("- **同工同酬**：相同岗位、相同工作内容、相同绩效——应获得相同报酬\n");
        result.push_str("- **性骚扰**：不受欢迎的性暗示、要求、言语或身体接触——构成违法的职场歧视\n");
        result.push_str("- **报复保护**：因投诉歧视而受到报复（降职、辞退、排挤）——本身也是违法行为\n");
        result.push_str("- **举证责任**：劳动者提出初步证据后，举证责任转移给雇主\n");
        result.push_str("- **ILO第100号和第111号公约**：同工同酬、消除就业和职业歧视\n");
    }

    if query_lower.contains("合同") || query_lower.contains("竞业") || query_lower.contains("试用") {
        result.push_str("### 劳动合同通用原则\n\n");
        result.push_str("- **书面合同**：劳动关系建立后应在法定期限内签订书面劳动合同\n");
        result.push_str("- **试用期**：通常1-6个月（各国不同），试用期工资不得低于正式工资的80%\n");
        result.push_str("- **竞业限制**：仅限于高管、高级技术人员和负有保密义务的人员，期限不超过2年，雇主须支付补偿金\n");
        result.push_str("- **合同变更**：变更合同内容需双方协商一致并书面确认\n");
        result.push_str("- **无固定期限合同**：连续工作满10年或连续签订两次固定期限合同后，劳动者有权要求签订无固定期限合同\n");
        result.push_str("- **违约金限制**：除竞业限制和培训服务期外，雇主不得约定由劳动者承担的违约金\n");
    }

    if result.len() < 200 {
        result.push_str("### 通用劳动权益\n\n");
        result.push_str("- **ILO核心公约**：结社自由（第87号）、集体谈判权（第98号）、禁止强迫劳动（第29/105号）、\n");
        result.push_str("  消除童工（第138/182号）、消除就业歧视（第100/111号）、职业安全健康（第155号）\n");
        result.push_str("- **维权途径**：劳动仲裁（通常免费或低费用）、劳动监察投诉、法院诉讼、工会协商\n");
        result.push_str("- **时效**：劳动争议申请仲裁的时效通常为1年（从知道权利被侵害之日算起）\n");
    }

    result.push_str("\n> 💡 如需更详细的信息，可以：\n");
    result.push_str("> 1. 使用 web_search 工具搜索当地最新的劳动法规\n");
    result.push_str("> 2. 联系当地劳动监察部门或工会\n");
    result.push_str("> 3. 咨询专业劳动法律师\n");

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_severance_illegal_dismissal_2n() {
        let tool = CalculateSeveranceTool;
        let args = serde_json::json!({
            "years_of_service": 5.0,
            "monthly_salary": 8000.0,
            "termination_reason": "illegal"
        });
        let result = tool.execute(&args.to_string()).await.unwrap();
        // 5 years = N=5, 2N = 8000 * 5 * 2 = 80000
        assert!(result.contains("80000"), "2N should be 80000 for 5yr/8k: {}", result);
        assert!(result.contains("违法解除"), "Should mention illegal dismissal");
    }

    #[tokio::test]
    async fn test_severance_no_fault_n_plus_1() {
        let tool = CalculateSeveranceTool;
        let args = serde_json::json!({
            "years_of_service": 3.5,
            "monthly_salary": 10000.0,
            "termination_reason": "no_fault"
        });
        let result = tool.execute(&args.to_string()).await.unwrap();
        // 3.5 years = N=3.5, N+1 = 10000*3.5+10000 = 45000
        assert!(result.contains("45000"), "N+1 should be 45000 for 3.5yr/10k: {}", result);
    }

    #[tokio::test]
    async fn test_severance_short_tenure() {
        let tool = CalculateSeveranceTool;
        let args = serde_json::json!({
            "years_of_service": 0.4,
            "monthly_salary": 5000.0,
            "termination_reason": "illegal"
        });
        let result = tool.execute(&args.to_string()).await.unwrap();
        // 0.4 years < 0.5 = N=0.5, 2N = 5000 * 0.5 * 2 = 5000
        assert!(result.contains("5000"), "2N should be 5000 for 0.4yr/5k: {}", result);
    }

    #[tokio::test]
    async fn test_severance_salary_cap() {
        let tool = CalculateSeveranceTool;
        let args = serde_json::json!({
            "years_of_service": 10.0,
            "monthly_salary": 50000.0,
            "termination_reason": "layoff",
            "local_avg_salary_3x": 30000.0
        });
        let result = tool.execute(&args.to_string()).await.unwrap();
        // N = 10, capped at 30000*10 = 300000
        assert!(result.contains("300000"), "Capped N should be 300000 for 10yr/50k cap 30k: {}", result);
        assert!(result.contains("封顶"), "Should mention salary cap");
    }

    #[tokio::test]
    async fn test_evidence_checklist_dismissal() {
        let tool = EvidenceChecklistTool;
        let args = serde_json::json!({
            "case_type": "dismissal",
            "specific_details": "以不能胜任为由辞退"
        });
        let result = tool.execute(&args.to_string()).await.unwrap();
        assert!(result.contains("解除劳动合同通知书"), "Should include dismissal notice");
        assert!(result.contains("绩效考核"), "Should include performance review for capability-based dismissal");
    }

    #[tokio::test]
    async fn test_labor_law_search_builtin() {
        let tool = LaborLawSearchTool::new(std::path::PathBuf::from("/tmp/nonexistent"));
        let args = serde_json::json!({
            "query": "辞退补偿"
        });
        let result = tool.execute(&args.to_string()).await.unwrap();
        assert!(result.contains("辞退") || result.contains("补偿"), "Should return builtin knowledge for dismissal");
        assert!(result.contains("N"), "Should mention N compensation standard");
    }

    #[tokio::test]
    async fn test_labor_law_search_discrimination() {
        let tool = LaborLawSearchTool::new(std::path::PathBuf::from("/tmp/nonexistent"));
        let args = serde_json::json!({
            "query": "职场歧视"
        });
        let result = tool.execute(&args.to_string()).await.unwrap();
        assert!(result.contains("歧视"), "Should return builtin knowledge for discrimination");
        assert!(result.contains("ILO"), "Should reference ILO conventions");
    }
}
