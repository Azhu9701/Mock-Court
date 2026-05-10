import Link from "next/link";
import { Brain, History, Search, BarChart3, ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";

const QUICK_LINKS = [
  { href: "/possess", label: "讨论", desc: "召唤思想家之魂，进行多视角分析", icon: Brain },
  { href: "/souls", label: "魂览", desc: "浏览思想家的魂档案", icon: Search },
  { href: "/sessions", label: "会话历史", desc: "回顾过去的附体会话", icon: History },
  { href: "/analytics", label: "仪表盘", desc: "万民幡运行数据概览", icon: BarChart3 },
];

export default function Home() {
  return (
    <div className="max-w-2xl mx-auto py-16 space-y-12">
      <div className="text-center space-y-4">
        <h1 className="text-4xl font-bold tracking-tight">万民幡</h1>
        <p className="text-lg text-muted-foreground max-w-md mx-auto">
          24 位思想家的 AI 对话实验室。召唤、合议、辩论——知识不在魂里，在实践里。
        </p>
        <Link href="/possess">
          <Button size="lg" className="mt-4">
            <Brain className="mr-2 h-5 w-5" />
            开始讨论
          </Button>
        </Link>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        {QUICK_LINKS.map((link) => {
          const Icon = link.icon;
          return (
            <Link key={link.href} href={link.href} className="group rounded-xl border p-5 hover:bg-muted/50 transition-colors">
              <div className="flex items-center gap-3 mb-2">
                <Icon className="h-5 w-5 text-primary" />
                <h3 className="font-semibold">{link.label}</h3>
                <ArrowRight className="h-4 w-4 ml-auto opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all" />
              </div>
              <p className="text-sm text-muted-foreground">{link.desc}</p>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
