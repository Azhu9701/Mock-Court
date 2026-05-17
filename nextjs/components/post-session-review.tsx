"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Loader2 } from "lucide-react";
import { saveReview } from "@/lib/api";

interface PostSessionReviewProps {
  sessionId: string;
  onComplete: () => void;
}

export function PostSessionReview({ sessionId, onComplete }: PostSessionReviewProps) {
  const [practiceCommitment, setPracticeCommitment] = useState("");
  const [emptyChair, setEmptyChair] = useState("");
  const [selfNegation, setSelfNegation] = useState("");
  const [saving, setSaving] = useState(false);

  const effectiveness = (() => {
    const c = practiceCommitment.trim();
    if (!c || c.length < 10) return "invalid";
    const vague = /了解|学习|思考|知道|认识|明白|关注|注意|研究一下|看看|想想/;
    if (vague.test(c) && c.length < 30) return "invalid";
    return c.length >= 30 ? "effective" : "partial";
  })();

  const handleComplete = async () => {
    setSaving(true);
    try {
      await saveReview(sessionId, {
        practice_commitment: practiceCommitment,
        empty_chair: emptyChair,
        self_negation: selfNegation,
        effectiveness,
        effectiveness_note:
          effectiveness === "invalid" ? "消费型"
          : effectiveness === "partial" ? "意向型"
          : "实践型",
      });
    } catch {} finally {
      setSaving(false);
      onComplete();
    }
  };

  return (
    <div className="space-y-5" data-testid="post-session-review">
      <div>
        <label className="text-sm font-medium">这次对话后你会做什么？</label>
        <Textarea
          value={practiceCommitment}
          onChange={(e) => setPracticeCommitment(e.target.value)}
          rows={2}
          className="mt-1"
        />
      </div>

      <div>
        <label className="text-sm font-medium">谁没被邀请但应在场？</label>
        <Textarea
          value={emptyChair}
          onChange={(e) => setEmptyChair(e.target.value)}
          rows={2}
          className="mt-1"
        />
      </div>

      <div>
        <label className="text-sm font-medium">什么判断被证明是错的？</label>
        <Textarea
          value={selfNegation}
          onChange={(e) => setSelfNegation(e.target.value)}
          rows={2}
          className="mt-1"
        />
      </div>

      <Button onClick={handleComplete} disabled={saving} className="w-full">
        {saving ? <Loader2 className="h-4 w-4 animate-spin mr-1" /> : null}
        完成闭环
      </Button>
    </div>
  );
}
