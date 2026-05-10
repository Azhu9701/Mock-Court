import { useRef, useMemo } from "react";

export function useCleanContent(content: string): string {
  const prevContentRef = useRef<string | null>(null);
  const cachedRef = useRef<string | null>(null);

  return useMemo(() => {
    if (!content) return "";
    if (prevContentRef.current === content && cachedRef.current !== null) {
      return cachedRef.current;
    }
    const cleaned = content.replace(/<[^>]+>/g, "");
    prevContentRef.current = content;
    cachedRef.current = cleaned;
    return cleaned;
  }, [content]);
}
