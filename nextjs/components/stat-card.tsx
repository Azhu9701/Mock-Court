import { BarChart3, Users, CheckCircle, AlertTriangle } from "lucide-react";
import { cn } from "@/lib/utils";

interface StatCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: string;
}

const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
  "bar-chart": BarChart3,
  users: Users,
  "check-circle": CheckCircle,
  "alert-triangle": AlertTriangle,
};

export function StatCard({ title, value, subtitle, icon }: StatCardProps) {
  const Icon = iconMap[icon];
  return (
    <div
      className="flex items-center gap-4 rounded-xl border bg-background p-6"
      data-testid={`stat-card-${title}`}
    >
      {Icon && (
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
          <Icon className="h-5 w-5 text-primary" />
        </div>
      )}
      <div>
        <p className="text-2xl font-bold">{value}</p>
        <p className="text-sm text-muted-foreground">{title}</p>
        {subtitle && <p className="text-xs text-muted-foreground mt-0.5">{subtitle}</p>}
      </div>
    </div>
  );
}
