"use client";

import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Brain, CheckCircle2, AlertCircle } from "lucide-react";

interface PracticeOpeningDialogProps {
  open: boolean;
  onStart: (judgment: string, worry: string, unknown: string) => void;
  onCancel: () => void;
}

export function PracticeOpeningDialog({ open, onStart, onCancel }: PracticeOpeningDialogProps) {
  const [judgment, setJudgment] = useState("");
  const [worry, setWorry] = useState("");
  const [unknown, setUnknown] = useState("");
  const [showHint, setShowHint] = useState(false);

  const canProceed = judgment.trim().length > 0;

  const handleStart = () => {
    if (!canProceed) {
      setShowHint(true);
      return;
    }
    onStart(judgment.trim(), worry.trim(), unknown.trim());
  };

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) onCancel(); }}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg">
            <AlertCircle className="h-5 w-5 text-orange-500" />
            实践开口
          </DialogTitle>
          <DialogDescription className="text-sm text-muted-foreground">
            检测到在场者经验，在进入 P1-P4 实践流程前，请先写下你的基本判断。
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3 mt-2">
          <div className="relative">
            <label className="text-xs font-medium text-muted-foreground">
              你的判断 <span className="text-red-500">*</span>
            </label>
            <Textarea
              value={judgment}
              onChange={(e) => { setJudgment(e.target.value); setShowHint(false); }}
              placeholder="对这个问题，你目前的基本判断是什么？"
              rows={2}
              className="mt-1 resize-none text-sm pr-6"
            />
            {judgment.trim() && (
              <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500 absolute top-7 right-2" />
            )}
          </div>

          <div className="relative">
            <label className="text-xs font-medium text-muted-foreground">你的担忧</label>
            <Textarea
              value={worry}
              onChange={(e) => setWorry(e.target.value)}
              placeholder="你最担心分析中可能忽略什么？"
              rows={2}
              className="mt-1 resize-none text-sm pr-6"
            />
            {worry.trim() && (
              <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500 absolute top-7 right-2" />
            )}
          </div>

          <div className="relative">
            <label className="text-xs font-medium text-muted-foreground">未知领域</label>
            <Textarea
              value={unknown}
              onChange={(e) => setUnknown(e.target.value)}
              placeholder="有哪些你不确定的关键信息或变量？"
              rows={2}
              className="mt-1 resize-none text-sm pr-6"
            />
            {unknown.trim() && (
              <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500 absolute top-7 right-2" />
            )}
          </div>

          {showHint && (
            <p className="text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1">
              <AlertCircle className="h-3 w-3" />
              请至少填写你的判断，这是实践开口的起点
            </p>
          )}
        </div>

        <div className="flex gap-2 justify-end mt-4">
          <Button variant="ghost" size="sm" onClick={onCancel} className="text-muted-foreground">
            取消
          </Button>
          <Button size="sm" onClick={handleStart} className="bg-orange-600 hover:bg-orange-700">
            <Brain className="mr-1.5 h-4 w-4" />
            开始实践
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
