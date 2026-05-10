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
import { deleteSoul } from "@/lib/api";

interface DeleteSoulConfirmDialogProps {
  soulName: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDeleted: () => void;
}

export function DeleteSoulConfirmDialog({
  soulName,
  open,
  onOpenChange,
  onDeleted,
}: DeleteSoulConfirmDialogProps) {
  const [deleting, setDeleting] = useState(false);

  const onConfirm = async () => {
    setDeleting(true);
    try {
      await deleteSoul(soulName);
      onDeleted();
    } finally {
      setDeleting(false);
      onOpenChange(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent data-testid="delete-soul-dialog">
        <DialogHeader>
          <DialogTitle>散魂确认</DialogTitle>
          <DialogDescription className="pt-2">
            魂曰：「{soulName}」将被散离，此操作不可撤回。
          </DialogDescription>
        </DialogHeader>
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button
            variant="destructive"
            onClick={onConfirm}
            disabled={deleting}
            data-testid="confirm-delete-btn"
          >
            {deleting ? "散离中..." : "散魂"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
