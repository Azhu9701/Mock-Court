import { KnowledgeSearch } from "@/components/knowledge-search";

export default function KnowledgePage() {
  return (
    <div className="flex flex-col gap-6 p-6 max-w-4xl mx-auto">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">知识库</h1>
        <p className="text-sm text-muted-foreground mt-1">
          全文检索所有魂分析输出、辩证综合和会话记录
        </p>
      </div>
      <KnowledgeSearch />
    </div>
  );
}
