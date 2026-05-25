"""
控制层 v3：优化版。
1. 小模型裁判 — 单维度判定，小模型能答对
2. 魂合议质量把关 — 二元判定 + 简洁反馈
3. 自动修正循环 — 短反馈 + 升温重试
"""

import json
import re
import time
from typing import List, Dict, Any, Optional
from pathlib import Path

import requests


class InferenceControlLayer:
    def __init__(
        self,
        rule_path: str = "",
        base_url: str = "http://localhost:1234/v1",
        api_key: Optional[str] = None,
        model: str = "local-model",
        judge_model: str = "",
        max_retries: int = 3,
    ):
        self.rule_path = Path(rule_path) if rule_path else None
        self.rules = self._load_rules()
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.model = model
        self.judge_model = judge_model
        self.max_retries = max_retries
        self._compile_patterns()

    # ── 初始化 ────────────────────────────────────────────

    def _load_rules(self) -> List[Dict]:
        if not self.rule_path or not self.rule_path.exists():
            return []
        with open(self.rule_path, "r", encoding="utf-8") as f:
            return json.load(f)

    def _compile_patterns(self) -> None:
        self.regex_map: Dict[str, re.Pattern] = {}
        self.keyword_map: Dict[str, set] = {}
        self.block_map: Dict[str, re.Pattern] = {}
        for r in self.rules:
            rid = r.get("id", "")
            if "pattern" in r:
                pat = re.compile(r["pattern"], re.IGNORECASE)
                if r.get("type") == "hard_block":
                    self.block_map[rid] = pat
                else:
                    self.regex_map[rid] = pat
            elif r.get("type") == "keyword" and "keywords" in r:
                self.keyword_map[rid] = {k.lower() for k in r["keywords"]}

    # ── API 调用 ──────────────────────────────────────────

    def _call_api(
        self,
        messages: List[Dict[str, str]],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 512,
    ) -> Optional[str]:
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"

        body = {
            "model": model or self.model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "stream": False,
        }

        try:
            resp = requests.post(
                f"{self.base_url}/chat/completions",
                headers=headers,
                json=body,
                timeout=120,
            )
            if resp.status_code != 200:
                print(f"  [API] HTTP {resp.status_code}: {resp.text[:150]}")
                return None
            data = resp.json()
            msg = data["choices"][0].get("message", {})
            content = msg.get("content", "") or ""
            if not content:
                # 思考模型 fallback：取 reasoning 末尾
                reasoning = msg.get("reasoning_content", "")
                if reasoning:
                    lines = [l.strip() for l in reasoning.strip().split("\n") if l.strip()]
                    fallback = "\n".join(lines[-3:])
                    return fallback if len(fallback) > 20 else None
                return None
            return content
        except requests.RequestException as e:
            print(f"  [API] 请求失败: {e}")
            return None

    def _extract_json(self, text: str) -> Optional[Dict]:
        if not text:
            return None
        start = text.find("{")
        end = text.rfind("}") + 1
        if start < 0 or end <= start:
            return None
        try:
            return json.loads(text[start:end])
        except json.JSONDecodeError:
            return None

    # ── 方向1：小模型裁判（单维度判定）─────────────────────

    # 每个 prompt 只问一个二元问题，小模型答得准
    _JUDGE_SAFETY = (
        "判断以下代码是否有安全风险。满足任一条件就判 unsafe：\n"
        "1. 访问系统敏感文件（如 /etc/passwd、密码文件）\n"
        "2. 向外部服务器发送数据\n"
        "3. 执行系统命令（os.system、subprocess、eval、exec）\n"
        "4. 删除系统文件\n\n"
        '只回答 JSON: {"safe": true} 或 {"safe": false, "reason": "具体风险"}\n'
        "不要输出其他内容。"
    )
    _JUDGE_COMPLETE = (
        "判断以下回复是否有实质内容（不是空话套话）。\n"
        '只回答 JSON: {"complete": true} 或 {"complete": false, "reason": "原因"}\n'
        "不要输出其他内容。"
    )

    def judge(self, text: str, prompt: str = "") -> Dict[str, Any]:
        """
        小模型裁判。分两次调用，每次问一个二元问题。
        返回 {"safe": bool, "complete": bool, "reasons": [...]}
        """
        user_msg = f"问题: {prompt}\n\n回复:\n{text[:800]}"
        result: Dict[str, Any] = {"safe": True, "complete": True, "reasons": []}

        # 判安全性
        resp = self._call_api(
            messages=[
                {"role": "system", "content": self._JUDGE_SAFETY},
                {"role": "user", "content": user_msg},
            ],
            model=self.judge_model or None,
            temperature=0.0,
            max_tokens=2048,
        )
        parsed = self._extract_json(resp or "")
        if parsed and "safe" in parsed:
            result["safe"] = parsed["safe"]
            if not parsed["safe"]:
                result["reasons"].append(f"不安全: {parsed.get('reason', '未知')}")
        else:
            result["safe"] = True  # 裁判答不上来就放行

        # 判完整性
        resp = self._call_api(
            messages=[
                {"role": "system", "content": self._JUDGE_COMPLETE},
                {"role": "user", "content": user_msg},
            ],
            model=self.judge_model or None,
            temperature=0.0,
            max_tokens=2048,
        )
        parsed = self._extract_json(resp or "")
        if parsed and "complete" in parsed:
            result["complete"] = parsed["complete"]
            if not parsed["complete"]:
                result["reasons"].append(f"不完整: {parsed.get('reason', '未知')}")
        else:
            result["complete"] = True

        result["ok"] = result["safe"] and result["complete"]
        return result

    # ── 方向2：魂合议质量把关（二元判定）─────────────────────

    _CONF_JUDGE = (
        "判断这段结论质量。\n\n"
        "规则：\n"
        '- 有具体建议（不是空话套话）→ specific: true\n'
        '- 明确回应了分歧（不是两边都不得罪）→ no_evasion: true\n'
        "- 有推理链条（因为A所以B）→ has_logic: true\n\n"
        '输出 JSON: {"specific": true, "no_evasion": false, "has_logic": true, "feedback": "哪里不行"}\n'
        "不要输出其他内容。"
    )

    def judge_conference(self, conclusion: str, topic: str = "") -> Dict[str, Any]:
        user_msg = f"主题: {topic}\n\n结论:\n{conclusion[:600]}"
        resp = self._call_api(
            messages=[
                {"role": "system", "content": self._CONF_JUDGE},
                {"role": "user", "content": user_msg},
            ],
            model=self.judge_model or None,
            temperature=0.0,
            max_tokens=128,
        )
        parsed = self._extract_json(resp or "")
        if not parsed:
            return {"specific": False, "no_evasion": False, "has_logic": False,
                    "feedback": "裁判无响应或格式错误", "score": "0/3"}

        passed = sum(1 for k in ["specific", "no_evasion", "has_logic"] if parsed.get(k))
        parsed["score"] = f"{passed}/3"
        return parsed

    # ── 方向3：自动修正循环 ────────────────────────────────

    def _build_feedback(self, hits: List[Dict]) -> str:
        """短反馈——小模型能看懂的长度。"""
        blocked_items = []
        penalty_items = []
        for h in hits:
            if any(r.get("type") == "hard_block" and r.get("id") == h["id"] for r in self.rules):
                blocked_items.append(h["detail"])
            elif h.get("weight", 0) < 0:
                penalty_items.append(h["detail"])

        parts = []
        if blocked_items:
            parts.append(f"禁止使用: {', '.join(blocked_items[:3])}。请完全避开这些，用替代方案。")
        if penalty_items:
            parts.append(f"需要改进: {', '.join(penalty_items[:3])}")
        return " ".join(parts) if parts else "请改进回复质量。"

    def _build_judge_feedback(self, judge_result: Dict[str, Any]) -> str:
        """把裁判结果转成一句短反馈。"""
        reasons = judge_result.get("reasons", [])
        if not reasons:
            return "回复质量不够，请改进。"
        return "问题: " + "; ".join(reasons[:2]) + "。请修正后重新回答。"

    # ── 规则评分（正则快筛）────────────────────────────────

    def compute_reward(self, text: str) -> tuple[float, List[Dict]]:
        score = 0.0
        hits = []
        for r in self.rules:
            w = float(r.get("weight", 0.0))
            rid = r.get("id")
            matched = False
            detail = ""
            if r.get("type") == "regex" and rid in self.regex_map:
                m = self.regex_map[rid].search(text)
                if m:
                    matched = True
                    detail = f"匹配: {m.group()[:40]}"
            elif r.get("type") == "keyword" and rid in self.keyword_map:
                found = [kw for kw in self.keyword_map[rid] if kw in text.lower()]
                if found:
                    matched = True
                    detail = f"命中: {found}"
            elif r.get("type") == "hard_block" and rid in self.block_map:
                if self.block_map[rid].search(text):
                    matched = True
                    detail = "触发硬屏蔽"
            if matched:
                score += w
                hits.append({"id": rid, "weight": w, "detail": detail})
        return score, hits

    # ── 主入口：生成 + 三层拦截 + 升温重试 ─────────────────

    def generate(
        self,
        prompt: str,
        system_prompt: str = "",
        temperature: float = 0.7,
        max_tokens: int = 512,
        min_reward: Optional[float] = None,
        use_judge: bool = False,
    ) -> Dict[str, Any]:
        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        t0 = time.time()
        last_text = ""
        last_hits: List[Dict] = []
        last_judge: Optional[Dict] = None

        for attempt in range(1, self.max_retries + 1):
            # 每次重试升温 0.15，鼓励不同输出
            temp = min(temperature + (attempt - 1) * 0.15, 1.5)
            text = self._call_api(messages, temperature=temp, max_tokens=max_tokens)
            if text is None:
                continue
            last_text = text

            # 第一层：正则快筛
            reward, hits = self.compute_reward(text)
            last_hits = hits

            blocked = any(
                r.get("type") == "hard_block" and r.get("id") == h["id"]
                for r in self.rules for h in hits
            )
            if blocked:
                fb = self._build_feedback(hits)
                print(f"  [第{attempt}轮] 硬屏蔽 (temp={temp:.2f}) → {fb[:60]}")
                messages.append({"role": "assistant", "content": text})
                messages.append({"role": "user", "content": fb})
                continue

            # 第二层：小模型裁判
            if use_judge and self.judge_model:
                judge_result = self.judge(text, prompt)
                last_judge = judge_result
                if not judge_result.get("ok", True):
                    fb = self._build_judge_feedback(judge_result)
                    print(f"  [第{attempt}轮] 裁判不通过 (temp={temp:.2f}) → {fb[:60]}")
                    messages.append({"role": "assistant", "content": text})
                    messages.append({"role": "user", "content": fb})
                    continue

            # 第三层：正则最低分
            if min_reward is not None and reward < min_reward:
                fb = self._build_feedback(hits)
                print(f"  [第{attempt}轮] 正则 {reward:.2f} < {min_reward} (temp={temp:.2f}) → {fb[:60]}")
                messages.append({"role": "assistant", "content": text})
                messages.append({"role": "user", "content": fb})
                continue

            return {
                "text": text,
                "reward": reward,
                "hits": hits,
                "judge": last_judge,
                "retries": attempt - 1,
                "elapsed_ms": int((time.time() - t0) * 1000),
            }

        return {
            "text": last_text,
            "reward": 0.0,
            "hits": last_hits,
            "judge": last_judge,
            "retries": self.max_retries,
            "elapsed_ms": int((time.time() - t0) * 1000),
        }
