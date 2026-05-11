"use client";

import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { saveVerificationKnowledgeCard } from "@/lib/api";
import { Clock, Send, CheckCircle, Loader2, AlertCircle } from "lucide-react";

interface VerificationDialogProps {
  open: boolean;
  sessionId: string;
  sessionTitle: string;
  onComplete: () => void;
  onClose: () => void;
}

function FieldGroup({
  label,
  icon: Icon,
  value,
  onChange,
  placeholder,
  required,
}: {
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
  required?: boolean;
}) {
  return (
    <div className="space-y-1.5">
      <label className="text-xs font-medium text-muted-foreground flex items-center gap-1.5">
        <Icon className="h-3.5 w-3.5" />
        {label}
        {required && <span className="text-red-500">*</span>}
      </label>
      <Textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        rows={3}
        className="resize-none text-sm"
      />
    </div>
  );
}

export function VerificationDialog({
  open,
  sessionId,
  sessionTitle,
  onComplete,
  onClose,
}: VerificationDialogProps) {
  const [action, setAction] = useState("");
  const [validSignal, setValidSignal] = useState("");
  const [revisionSignal, setRevisionSignal] = useState("");
  const [showHint, setShowHint] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const canSubmit = action.trim().length > 0;

  const handleSubmit = async () => {
    if (!canSubmit) {
      setShowHint(true);
      return;
    }

    setSaving(true);
    try {
      await saveVerificationKnowledgeCard({
        session_id: sessionId,
        title: `实践检验 · ${sessionTitle.slice(0, 30)}`,
        action: action.trim(),
        valid_signal: validSignal.trim(),
        revision_signal: revisionSignal.trim(),
      });
      setSaved(true);
      setTimeout(() => {
        onComplete();
        resetForm();
      }, 2000);
    } catch {
    } finally {
      setSaving(false);
    }
  };

  const resetForm = () => {
    setAction("");
    setValidSignal("");
    setRevisionSignal("");
    setShowHint(false);
    setSaved(false);
  };

  const handleClose = () => {
    if (!saved) {
      resetForm();
    }
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) handleClose(); }}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg">
            <Clock className="h-5 w-5 text-blue-500" />
            24小时可检验项
          </DialogTitle>
          <DialogDescription className="text-sm text-muted-foreground space-y-2">
            <p>
              综合以上分析，请设定一个<strong>在未来24小时内可以实际检验的具体行动</strong>：
            </p>
            <ul className="list-disc list-inside space-y-1 text-xs">
              <li>你准备做什么来验证（或挑战）这次分析的结论？</li>
              <li>你预计什么信号表示"分析有效"？什么信号表示"分析需要修正"？</li>
              <li>检验后，请在下次附体时通过实践开口带回现场数据。</li>
            </ul>
            <p className="text-xs text-amber-600 dark:text-amber-400 font-medium">
              如果24小时内不检验——本次分析的结论标记为"待验证"而非"已确认"。
            </p>
          </DialogDescription>
        </DialogHeader>

        {saved ? (
          <div className="flex flex-col items-center justify-center gap-3 py-8">
            <CheckCircle className="h-12 w-12 text-emerald-500" />
            <p className="text-sm font-medium text-emerald-600 dark:text-emerald-400">
              检验项已记录到知识卡片
            </p>
          </div>
        ) : (
          <>
            <div className="space-y-4">
              <FieldGroup
                label="检验行动"
                icon={Send}
                required
                value={action}
                onChange={(v) => { setAction(v); setShowHint(false); }}
                placeholder="你准备做什么来验证这次分析的结论？例如：去实地观察、查阅某份资料、询问某个相关者…"
              />
              <FieldGroup
                label="有效信号"
                icon={CheckCircle}
                value={validSignal}
                onChange={setValidSignal}
                placeholder="什么现象或数据表示'分析有效'？"
              />
              <FieldGroup
                label="修正信号"
                icon={AlertCircle}
                value={revisionSignal}
                onChange={setRevisionSignal}
                placeholder="什么现象或数据表示'分析需要修正'？"
              />

              {showHint && (
                <p className="text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1">
                  <AlertCircle className="h-3 w-3" />
                  请至少填写检验行动
                </p>
              )}
            </div>

            <div className="flex gap-2 justify-end mt-2">
              <Button variant="ghost" size="sm" onClick={handleClose} className="text-muted-foreground">
                跳过
              </Button>
              <Button size="sm" onClick={handleSubmit} disabled={saving}>
                {saving ? (
                  <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                ) : (
                  <Send className="mr-1.5 h-4 w-4" />
                )}
                记录检验项
              </Button>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
