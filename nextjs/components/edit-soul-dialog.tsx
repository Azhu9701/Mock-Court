"use client";

import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { SoulProfile } from "@/lib/api";
import { updateSoul } from "@/lib/api";

const formSchema = z.object({
  ismism_code: z.string().regex(/^\d-\d-\d-\d$/, "格式: 1-2-3-3"),
  field: z.string().min(1, "必填"),
  domains: z.string().optional(),
  tags: z.string().optional(),
  summon_prompt: z.string().min(1, "必填"),
});

type FormValues = z.infer<typeof formSchema>;

interface EditSoulDialogProps {
  soul: SoulProfile;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSaved: () => void;
}

export function EditSoulDialog({
  soul,
  open,
  onOpenChange,
  onSaved,
}: EditSoulDialogProps) {
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      ismism_code: soul.ismism_code,
      field: soul.field,
      domains: soul.domains.join(", "),
      summon_prompt: soul.summon_prompt,
    },
  });

  const onSubmit = async (values: FormValues) => {
    await updateSoul(soul.name, {
      ismism_code: values.ismism_code,
      field: values.field,
      domains: values.domains
        ? values.domains.split(",").map((s: string) => s.trim())
        : [],
      tags: values.tags
        ? values.tags.split(",").map((s: string) => s.trim())
        : [],
      summon_prompt: values.summon_prompt,
    });
    onSaved();
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent data-testid="edit-soul-dialog">
        <DialogHeader>
          <DialogTitle>编辑 {soul.name}</DialogTitle>
        </DialogHeader>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="text-sm font-medium">Ismism 编码</label>
            <Input {...form.register("ismism_code")} placeholder="1-2-3-3" />
            {form.formState.errors.ismism_code && (
              <p className="text-xs text-red-500 mt-1">
                {form.formState.errors.ismism_code.message}
              </p>
            )}
          </div>
          <div>
            <label className="text-sm font-medium">领域</label>
            <Input {...form.register("field")} />
          </div>
          <div>
            <label className="text-sm font-medium">领域标签 (逗号分隔)</label>
            <Input {...form.register("domains")} />
          </div>
          <div>
            <label className="text-sm font-medium">标签 (逗号分隔)</label>
            <Input {...form.register("tags")} />
          </div>
          <div>
            <label className="text-sm font-medium">召唤词</label>
            <textarea
              {...form.register("summon_prompt")}
              rows={6}
              className="w-full mt-1 rounded-md border px-3 py-2 text-sm font-mono"
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit" data-testid="save-soul-btn">
              保存
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
