"use client";

import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ChevronDown, ChevronUp, Pencil, Save, X, Loader2 } from "lucide-react";
import { updateSoul } from "@/lib/api";

interface SoulPromptProps {
  prompt: string;
  soulName?: string;
}

export function SoulPrompt({ prompt, soulName }: SoulPromptProps) {
  const [expanded, setExpanded] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editValue, setEditValue] = useState(prompt);
  const [saving, setSaving] = useState(false);
  const [currentPrompt, setCurrentPrompt] = useState(prompt);

  const startEdit = () => {
    setEditValue(currentPrompt);
    setEditing(true);
  };

  const cancelEdit = () => {
    setEditing(false);
  };

  const saveEdit = async () => {
    if (!soulName || saving) return;
    setSaving(true);
    try {
      await updateSoul(soulName, { summon_prompt: editValue });
      setCurrentPrompt(editValue);
      setEditing(false);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div data-testid="soul-prompt">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-semibold">召唤词</h3>
        <div className="flex items-center gap-1">
          {editing ? (
            <>
              <Button
                variant="ghost"
                size="sm"
                onClick={cancelEdit}
                disabled={saving}
                data-testid="prompt-cancel-edit"
              >
                <X className="h-3 w-3 mr-1" />取消
              </Button>
              <Button
                variant="default"
                size="sm"
                onClick={saveEdit}
                disabled={saving}
                data-testid="prompt-save-edit"
              >
                {saving ? (
                  <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                ) : (
                  <Save className="h-3 w-3 mr-1" />
                )}
                保存
              </Button>
            </>
          ) : (
            <>
              {soulName && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={startEdit}
                  data-testid="prompt-edit-btn"
                >
                  <Pencil className="h-3 w-3 mr-1" />编辑
                </Button>
              )}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setExpanded(!expanded)}
                data-testid="prompt-toggle"
              >
                {expanded ? (
                  <>
                    <ChevronUp className="h-3 w-3 mr-1" />收起
                  </>
                ) : (
                  <>
                    <ChevronDown className="h-3 w-3 mr-1" />展开
                  </>
                )}
              </Button>
            </>
          )}
        </div>
      </div>

      {editing ? (
        <Textarea
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
          rows={20}
          className="font-mono text-sm"
          data-testid="prompt-edit-textarea"
        />
      ) : (
        <div
          className={`overflow-hidden transition-all ${
            expanded ? "max-h-[2000px]" : "max-h-24"
          }`}
        >
          <div className="rounded-md bg-muted p-3 prose prose-sm max-w-none dark:prose-invert
            [&_h1]:text-base [&_h1]:font-bold [&_h1]:mt-3 [&_h1]:mb-2
            [&_h2]:text-sm [&_h2]:font-semibold [&_h2]:mt-3 [&_h2]:mb-1.5
            [&_h3]:text-sm [&_h3]:font-semibold [&_h3]:mt-3 [&_h3]:mb-1.5
            [&_p]:my-1.5 [&_p]:leading-relaxed [&_p]:text-sm
            [&_ul]:my-1 [&_ol]:my-1
            [&_li]:my-1 [&_li]:text-sm [&_li]:leading-relaxed
            [&_blockquote]:my-1.5 [&_blockquote]:pl-3 [&_blockquote]:border-l-2 [&_blockquote]:border-primary/30 [&_blockquote]:text-muted-foreground
            [&_strong]:font-semibold [&_strong]:text-foreground/90
            [&_em]:italic
            [&_hr]:my-3 [&_hr]:border-border/50
            [&_code]:bg-muted-foreground/15 [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_code]:text-xs
            [&_pre]:my-2 [&_pre]:p-3 [&_pre]:bg-muted-foreground/10 [&_pre]:rounded-lg [&_pre]:text-xs [&_pre]:overflow-x-auto
            [&_a]:text-primary [&_a]:underline [&_a]:underline-offset-2
          ">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {currentPrompt}
            </ReactMarkdown>
          </div>
        </div>
      )}
    </div>
  );
}
