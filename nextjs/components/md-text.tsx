import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

/** 过滤 AI thinking / reasoning 内容 */
export function stripThinking(text: string): string {
  if (!text) return "";
  return text
    .replace(/Here's a thinking process:[\s\S]*?(?=\n\n[A-Z]|\n#[^#]|$)/gi, "")
    .replace(/<thinking>[\s\S]*?<\/thinking>/gi, "")
    .replace(/Thinking:[\s\S]*?(?=\n\n[A-Z]|$)/gi, "")
    .trim();
}

/** Markdown 渲染容器，复用 prose 样式 */
export function MdText({
  children,
  className = "",
}: {
  children: string;
  className?: string;
}) {
  const cleaned = stripThinking(children);
  if (!cleaned) return null;
  return (
    <span
      className={`prose prose-slate prose-sm max-w-none [&_p]:my-1 [&_strong]:font-semibold [&_ul]:my-1 [&_ol]:my-1 [&_li]:my-0.5 ${className}`}
    >
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{cleaned}</ReactMarkdown>
    </span>
  );
}
