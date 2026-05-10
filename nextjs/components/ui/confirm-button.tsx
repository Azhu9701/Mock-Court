"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";

interface ConfirmButtonProps {
  onConfirm: () => Promise<void> | void;
  confirmText?: string;
  cancelText?: string;
  variant?: "destructive" | "ghost";
  size?: "sm" | "icon";
  icon: React.ReactNode;
  title?: string;
  className?: string;
}

export function ConfirmButton({
  onConfirm,
  confirmText = "确认",
  cancelText = "取消",
  variant = "ghost",
  size = "icon",
  icon,
  title,
  className,
}: ConfirmButtonProps) {
  const [confirming, setConfirming] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleConfirm = async () => {
    setLoading(true);
    setError("");
    try {
      await onConfirm();
      setConfirming(false);
    } catch (e: any) {
      const errorMsg = e?.message || e?.toString() || "操作失败";
      setError(errorMsg);
      setConfirming(false);
    } finally {
      setLoading(false);
    }
  };

  if (confirming) {
    return (
      <div className={`flex items-center gap-1 ${className || ""}`}>
        <Button size="sm" variant={variant === "ghost" ? "destructive" : variant} onClick={handleConfirm} disabled={loading}>
          {loading ? <Loader2 className="h-3 w-3 animate-spin" /> : confirmText}
        </Button>
        <Button size="sm" variant="ghost" onClick={() => setConfirming(false)}>
          {cancelText}
        </Button>
      </div>
    );
  }

  return (
    <div className="relative">
      <Button
        variant={variant}
        size={size}
        onClick={() => setConfirming(true)}
        title={title}
        className={className}
      >
        {icon}
      </Button>
      {error && (
        <div className="absolute right-0 top-full mt-1 px-2 py-1 bg-red-500 text-white text-xs rounded whitespace-nowrap z-50">
          {error}
        </div>
      )}
    </div>
  );
}
