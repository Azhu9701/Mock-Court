"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { CheckCircle, Lightbulb, AlertCircle, Users } from "lucide-react";

interface PostSessionReviewProps {
  sessionId: string;
  onComplete: () => void;
}

export function PostSessionReview({ sessionId, onComplete }: PostSessionReviewProps) {
  const [step, setStep] = useState<"participation" | "negation" | "chair" | "effectiveness" | "done">("participation");
  const [mostUnexpected, setMostUnexpected] = useState("");
  const [alreadyKnown, setAlreadyKnown] = useState("");
  const [selfNegation, setSelfNegation] = useState("");
  const [emptyChair, setEmptyChair] = useState("");
  const [effectiveness, setEffectiveness] = useState<"effective" | "partial" | "invalid" | null>(null);
  const [effectivenessNote, setEffectivenessNote] = useState("");

  return (
    <div className="max-w-2xl mx-auto space-y-6 p-4" data-testid="post-session-review">
      <div>
        <h2 className="text-lg font-semibold">附体完成 — 反馈闭环</h2>
        <p className="text-sm text-muted-foreground mt-1">
          知识不在魂里，在实践里。你的反馈帮助系统校准匹配和学习。
        </p>
      </div>

      {/* Step 1: 使用者参与 */}
      {step === "participation" && (
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Lightbulb className="h-5 w-5 text-yellow-500" />
            <h3 className="font-semibold">使用者参与</h3>
          </div>
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium">最没想到的是什么？</label>
              <Textarea
                value={mostUnexpected}
                onChange={(e) => setMostUnexpected(e.target.value)}
                placeholder="哪个视角或结论是你之前没考虑到的？"
                rows={2}
                data-testid="unexpected-input"
              />
            </div>
            <div>
              <label className="text-sm font-medium">早就知道的是什么？</label>
              <Textarea
                value={alreadyKnown}
                onChange={(e) => setAlreadyKnown(e.target.value)}
                placeholder="哪些分析和你的直觉或经验一致？"
                rows={2}
                data-testid="known-input"
              />
            </div>
          </div>
          <Button onClick={() => setStep("negation")} className="w-full" data-testid="participation-done-btn">
            下一步：自我否定
          </Button>
        </div>
      )}

      {/* Step 2: 自我否定 */}
      {step === "negation" && (
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <AlertCircle className="h-5 w-5 text-red-500" />
            <h3 className="font-semibold">自我否定</h3>
          </div>
          <div>
            <label className="text-sm font-medium">哪个预设被动摇了？</label>
            <p className="text-xs text-muted-foreground mb-2">
              回顾你在附体前记录的判断和担忧——经过魂的分析后，哪个被证明不完全正确？
            </p>
            <Textarea
              value={selfNegation}
              onChange={(e) => setSelfNegation(e.target.value)}
              placeholder="例：我原以为精益生产的问题是工人不愿改变，但马克思从劳动异化角度的分析让我重新思考..."
              rows={3}
              data-testid="negation-input"
            />
          </div>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setStep("participation")}>上一步</Button>
            <Button onClick={() => setStep("chair")} className="flex-1" data-testid="negation-done-btn">
              下一步：空椅子
            </Button>
          </div>
        </div>
      )}

      {/* Step 3: 空椅子 */}
      {step === "chair" && (
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Users className="h-5 w-5 text-purple-500" />
            <h3 className="font-semibold">空椅子</h3>
          </div>
          <div>
            <label className="text-sm font-medium">
              谁没有获得发言权，但应该有？
            </label>
            <p className="text-xs text-muted-foreground mb-2">
              哪些视角或人物在这场附体中被遗漏了？哪类人、哪种立场没有被代表？
            </p>
            <Textarea
              value={emptyChair}
              onChange={(e) => setEmptyChair(e.target.value)}
              placeholder="例：车间一线工人的视角没有被纳入..."
              rows={2}
              data-testid="chair-input"
            />
          </div>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setStep("negation")}>上一步</Button>
            <Button onClick={() => setStep("effectiveness")} className="flex-1" data-testid="chair-done-btn">
              下一步：有效性评分
            </Button>
          </div>
        </div>
      )}

      {/* Step 4: 有效性评分 */}
      {step === "effectiveness" && (
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <CheckCircle className="h-5 w-5 text-green-500" />
            <h3 className="font-semibold">有效性评分</h3>
          </div>
          <p className="text-sm text-muted-foreground">这次附体整体效果如何？</p>
          <div className="grid grid-cols-3 gap-3">
            <button
              onClick={() => setEffectiveness("effective")}
              className={`rounded-lg border-2 p-4 text-center transition-colors ${
                effectiveness === "effective"
                  ? "border-green-500 bg-green-50 dark:bg-green-950"
                  : "border-border hover:border-green-300"
              }`}
              data-testid="effective-btn"
            >
              <p className="font-semibold text-green-600">有效</p>
              <p className="text-xs text-muted-foreground mt-1">含独立盲区判断，被结论采纳</p>
            </button>
            <button
              onClick={() => setEffectiveness("partial")}
              className={`rounded-lg border-2 p-4 text-center transition-colors ${
                effectiveness === "partial"
                  ? "border-yellow-500 bg-yellow-50 dark:bg-yellow-950"
                  : "border-border hover:border-yellow-300"
              }`}
              data-testid="partial-btn"
            >
              <p className="font-semibold text-yellow-600">部分有效</p>
              <p className="text-xs text-muted-foreground mt-1">有参考价值但未实质改变结论</p>
            </button>
            <button
              onClick={() => setEffectiveness("invalid")}
              className={`rounded-lg border-2 p-4 text-center transition-colors ${
                effectiveness === "invalid"
                  ? "border-red-500 bg-red-50 dark:bg-red-950"
                  : "border-border hover:border-red-300"
              }`}
              data-testid="invalid-btn"
            >
              <p className="font-semibold text-red-600">无效</p>
              <p className="text-xs text-muted-foreground mt-1">不相关/事实错误/被推翻</p>
            </button>
          </div>
          {effectiveness && (
            <div>
              <label className="text-sm font-medium">评分理由</label>
              <Textarea
                value={effectivenessNote}
                onChange={(e) => setEffectivenessNote(e.target.value)}
                placeholder="简要说明评分原因..."
                rows={2}
              />
            </div>
          )}
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setStep("chair")}>上一步</Button>
            <Button
              onClick={() => setStep("done")}
              disabled={!effectiveness}
              className="flex-1"
              data-testid="effectiveness-done-btn"
            >
              完成反馈
            </Button>
          </div>
        </div>
      )}

      {/* Done */}
      {step === "done" && (
        <div className="flex flex-col items-center gap-4 py-10 text-center">
          <CheckCircle className="h-12 w-12 text-green-500" />
          <div>
            <h3 className="text-lg font-semibold">反馈闭环完成</h3>
            <p className="text-sm text-muted-foreground mt-1">
              你的反馈已记录。知识不在魂里，在实践里。
            </p>
          </div>
          <Button onClick={onComplete} data-testid="feedback-done-btn">返回</Button>
        </div>
      )}
    </div>
  );
}
