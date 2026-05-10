"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { DeleteSoulConfirmDialog } from "@/components/delete-soul-confirm-dialog";

interface DeleteSoulButtonProps {
  soulName: string;
  variant?: "icon" | "text";
  className?: string;
}

export function DeleteSoulButton({ soulName, variant = "icon", className }: DeleteSoulButtonProps) {
  const [open, setOpen] = useState(false);
  const router = useRouter();

  const handleDeleted = () => {
    router.push("/souls");
    router.refresh();
  };

  if (variant === "text") {
    return (
      <>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setOpen(true)}
          className={className}
        >
          <Trash2 className="h-4 w-4 mr-1" />
          散魂
        </Button>
        <DeleteSoulConfirmDialog
          soulName={soulName}
          open={open}
          onOpenChange={setOpen}
          onDeleted={handleDeleted}
        />
      </>
    );
  }

  return (
    <>
      <button
        onClick={(e) => {
          e.stopPropagation();
          e.preventDefault();
          setOpen(true);
        }}
        className="rounded-md p-1.5 hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
        title="散魂"
      >
        <Trash2 className="h-4 w-4" />
      </button>
      <DeleteSoulConfirmDialog
        soulName={soulName}
        open={open}
        onOpenChange={setOpen}
        onDeleted={handleDeleted}
      />
    </>
  );
}
