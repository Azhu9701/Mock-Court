"use client";

import { useState } from "react";
import { Settings2, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { updateSoul } from "@/lib/api";
import { DEEPSEEK_MODELS, REASONING_OPTIONS } from "@/config/models";

interface SoulModelConfigProps {
  soulName: string;
  currentModel: string;
}

export function SoulModelConfig({ soulName, currentModel }: SoulModelConfigProps) {
  const [editing, setEditing] = useState(false);
  const [model, setModel] = useState(currentModel || "");
  const [reasoning, setReasoning] = useState("");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      await updateSoul(soulName, { model, reasoning_effort: reasoning });
      setSaved(true);
      setEditing(false);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save model config:", e);
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    setModel(currentModel || "");
    setReasoning("");
    setEditing(false);
  };

  if (!editing) {
    return (
      <div className="flex items-center justify-between rounded-lg border bg-card p-4">
        <div className="flex items-center gap-3">
          <Settings2 className="h-4 w-4 text-muted-foreground" />
          <div>
            <p className="text-sm font-medium">模型配置</p>
            <p className="text-xs text-muted-foreground">
              当前模型：{currentModel || "使用默认配置"}
            </p>
          </div>
        </div>
        <Button variant="outline" size="sm" onClick={() => setEditing(true)}>
          自定义配置
        </Button>
      </div>
    );
  }

  return (
    <div className="rounded-lg border bg-card p-4 space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Settings2 className="h-4 w-4 text-muted-foreground" />
          <p className="text-sm font-medium">为 {soulName} 自定义模型配置</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={handleCancel}
            disabled={saving}
          >
            <X className="h-4 w-4" />
          </Button>
          <Button
            size="sm"
            variant="default"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "保存中..." : saved ? <Check className="h-4 w-4" /> : "保存"}
          </Button>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <label className="text-xs font-medium block mb-1.5">使用的模型</label>
          <Select value={model} onValueChange={(value) => setModel(value ?? "")}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="选择模型" />
            </SelectTrigger>
            <SelectContent>
              {DEEPSEEK_MODELS.map((m) => (
                <SelectItem key={m.value} value={m.value}>
                  {m.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-[10px] text-muted-foreground mt-1">
            空值表示使用全局默认配置
          </p>
        </div>

        <div>
          <label className="text-xs font-medium block mb-1.5">推理强度</label>
          <Select value={reasoning} onValueChange={(value) => setReasoning(value ?? "")}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="选择推理强度" />
            </SelectTrigger>
            <SelectContent>
              {REASONING_OPTIONS.map((r) => (
                <SelectItem key={r.value} value={r.value}>
                  {r.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  );
}
