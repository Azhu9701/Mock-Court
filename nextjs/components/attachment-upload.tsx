"use client";

import { useState, useRef, useCallback, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Paperclip, X, Loader2, FileImage, CheckCircle2, AlertCircle, Upload } from "lucide-react";
import { ocrFiles } from "@/lib/api";

interface AttachmentFile {
  id: string;
  file: File;
  previewUrl: string;
  ocrText: string | null;
  ocrStatus: "pending" | "ocr" | "done" | "error";
  ocrError?: string;
}

interface AttachmentUploadProps {
  onOcrResults: (texts: string[]) => void;
}

let idCounter = 0;
function uid() {
  return `att-${++idCounter}-${Date.now()}`;
}

function createPreview(file: File): string {
  return URL.createObjectURL(file);
}

export function AttachmentUpload({ onOcrResults }: AttachmentUploadProps) {
  const [files, setFiles] = useState<AttachmentFile[]>([]);
  const [ocrRunning, setOcrRunning] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dropRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState(false);

  const addFiles = useCallback((newFiles: File[]) => {
    const imageFiles = newFiles.filter((f) => f.type.startsWith("image/"));
    if (imageFiles.length === 0) return;
    setFiles((prev) => {
      const updated = [...prev];
      for (const f of imageFiles) {
        if (updated.length >= 5) break;
        updated.push({
          id: uid(),
          file: f,
          previewUrl: createPreview(f),
          ocrText: null,
          ocrStatus: "pending",
        });
      }
      return updated;
    });
  }, []);

  const removeFile = useCallback((id: string) => {
    setFiles((prev) => {
      const file = prev.find((f) => f.id === id);
      if (file) URL.revokeObjectURL(file.previewUrl);
      return prev.filter((f) => f.id !== id);
    });
  }, []);

  const startOcr = useCallback(async () => {
    const pending = files.filter((f) => f.ocrStatus === "pending");
    if (pending.length === 0) return;

    setOcrRunning(true);
    setFiles((prev) =>
      prev.map((f) =>
        pending.some((p) => p.id === f.id)
          ? { ...f, ocrStatus: "ocr" as const }
          : f
      )
    );

    try {
      const results = await ocrFiles(pending.map((f) => f.file));
      const texts: string[] = [];

      setFiles((prev) =>
        prev.map((f) => {
          const result = results.find((r) => r.filename === f.file.name);
          if (!result) return f;
          if (result.text) texts.push(`--- [${f.file.name}] ---\n${result.text}`);
          return {
            ...f,
            ocrStatus: result.error ? "error" : "done",
            ocrText: result.text,
            ocrError: result.error ?? undefined,
          };
        })
      );

      if (texts.length > 0) onOcrResults(texts);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : "OCR 失败";
      setFiles((prev) =>
        prev.map((f) =>
          f.ocrStatus === "ocr"
            ? { ...f, ocrStatus: "error" as const, ocrError: msg }
            : f
        )
      );
    } finally {
      setOcrRunning(false);
    }
  }, [files, onOcrResults]);

  // Paste handler
  useEffect(() => {
    const handler = (e: ClipboardEvent) => {
      const items = e.clipboardData?.items;
      if (!items) return;
      const imageFiles: File[] = [];
      for (let i = 0; i < items.length; i++) {
        const item = items[i];
        if (item.type.startsWith("image/")) {
          const file = item.getAsFile();
          if (file) imageFiles.push(file);
        }
      }
      if (imageFiles.length > 0) {
        e.preventDefault();
        addFiles(imageFiles);
      }
    };
    document.addEventListener("paste", handler);
    return () => document.removeEventListener("paste", handler);
  }, [addFiles]);

  // Drag & drop handlers
  const onDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragging(true);
  }, []);

  const onDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragging(false);
  }, []);

  const onDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragging(false);
      const dropped = Array.from(e.dataTransfer.files);
      addFiles(dropped);
    },
    [addFiles]
  );

  const pendingCount = files.filter((f) => f.ocrStatus === "pending").length;

  return (
    <div
      ref={dropRef}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      className={`rounded-xl border-2 border-dashed p-4 transition-colors ${
        dragging ? "border-primary bg-primary/5" : "border-border"
      } ${files.length > 0 ? "pb-3" : ""}`}
    >
      {files.length === 0 ? (
        <div className="flex flex-col items-center gap-2 py-4 text-center">
          <Paperclip className="h-8 w-8 text-muted-foreground/40" />
          <div className="text-sm text-muted-foreground">
            <span>拖拽图片到这里，或 </span>
            <button
              type="button"
              onClick={() => fileInputRef.current?.click()}
              className="text-primary underline underline-offset-2"
            >
              选择文件
            </button>
            <span>，或 Ctrl+V 粘贴截图</span>
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">
              {files.length} 个附件
            </span>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
              >
                <Upload className="h-3 w-3" />添加
              </button>
              {pendingCount > 0 && !ocrRunning && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={startOcr}
                >
                  <FileImage className="h-3 w-3 mr-1" />
                  开始识别 ({pendingCount})
                </Button>
              )}
              {ocrRunning && (
                <Button variant="outline" size="sm" disabled>
                  <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                  识别中...
                </Button>
              )}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-2">
            {files.map((f) => (
              <div
                key={f.id}
                className="relative rounded-lg border bg-background overflow-hidden group"
              >
                <button
                  type="button"
                  onClick={() => removeFile(f.id)}
                  className="absolute top-1 right-1 z-10 rounded-full bg-background/80 p-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  <X className="h-3.5 w-3.5" />
                </button>
                <img
                  src={f.previewUrl}
                  alt={f.file.name}
                  className="w-full h-24 object-cover"
                />
                <div className="p-2">
                  <p className="text-xs truncate text-muted-foreground mb-1">
                    {f.file.name}
                  </p>
                  {f.ocrStatus === "pending" && (
                    <span className="text-[10px] text-muted-foreground/60">
                      等待识别
                    </span>
                  )}
                  {f.ocrStatus === "ocr" && (
                    <span className="text-[10px] text-primary flex items-center gap-1">
                      <Loader2 className="h-2.5 w-2.5 animate-spin" />
                      识别中...
                    </span>
                  )}
                  {f.ocrStatus === "done" && f.ocrText && (
                    <span className="text-[10px] text-emerald-600 flex items-center gap-1">
                      <CheckCircle2 className="h-2.5 w-2.5" />
                      {f.ocrText.length} 字
                    </span>
                  )}
                  {f.ocrStatus === "done" && !f.ocrText && (
                    <span className="text-[10px] text-amber-600 flex items-center gap-1">
                      <AlertCircle className="h-2.5 w-2.5" />
                      无文字
                    </span>
                  )}
                  {f.ocrStatus === "error" && (
                    <span className="text-[10px] text-red-500 flex items-center gap-1">
                      <AlertCircle className="h-2.5 w-2.5" />
                      {f.ocrError || "识别失败"}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <input
        ref={fileInputRef}
        type="file"
        accept="image/png,image/jpeg,image/webp,image/gif"
        multiple
        className="hidden"
        onChange={(e) => {
          const selected = Array.from(e.target.files || []);
          addFiles(selected);
          e.target.value = "";
        }}
      />
    </div>
  );
}
