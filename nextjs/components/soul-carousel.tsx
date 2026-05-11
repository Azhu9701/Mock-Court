"use client";

import { useEffect, useState, useRef } from "react";
import { fetchSouls, type SoulListEntry } from "@/lib/api";
import { getSoulAvatarBg } from "@/lib/soul-utils";
import { Sparkles } from "lucide-react";

export function SoulCarousel() {
  const [souls, setSouls] = useState<SoulListEntry[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [visible, setVisible] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const timerRef = useRef<ReturnType<typeof setInterval>>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  useEffect(() => {
    fetchSouls()
      .then((list) => {
        const shuffled = [...list].sort(() => Math.random() - 0.5);
        setSouls(shuffled);
        setLoaded(true);
      })
      .catch(() => {
        setLoaded(true);
      });

    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, []);

  useEffect(() => {
    if (souls.length === 0) return;

    setVisible(true);

    timerRef.current = setInterval(() => {
      setVisible(false);
      timeoutRef.current = setTimeout(() => {
        if (!mountedRef.current) return;
        setCurrentIndex((prev) => (prev + 1) % souls.length);
        setVisible(true);
      }, 300);
    }, 1800);

    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, [souls.length]);

  const current = souls[currentIndex];

  if (!loaded) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="flex items-center gap-2 text-sm text-muted-foreground animate-pulse">
          <Sparkles className="h-4 w-4" />
          加载魂录...
        </div>
      </div>
    );
  }

  if (souls.length === 0) {
    return (
      <div className="text-center py-4 text-sm text-muted-foreground">
        暂无魂可用
      </div>
    );
  }

  return (
    <div className="flex items-center justify-center py-2">
      <div className="relative w-full max-w-xs">
        <div
          className="transition-all duration-300 ease-out"
          style={{
            opacity: visible ? 1 : 0,
            transform: visible ? "translateY(0) scale(1)" : "translateY(8px) scale(0.97)",
          }}
        >
          {current && (
            <div className="rounded-xl border bg-gradient-to-br from-background to-muted/30 p-4 shadow-sm">
              <div className="flex items-center gap-3">
                <div
                  className={`w-10 h-10 rounded-full flex items-center justify-center text-sm font-bold shrink-0 ${getSoulAvatarBg(current.ismism_code)}`}
                >
                  {current.name.charAt(0)}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-sm truncate">{current.name}</span>
                    <span className="text-[10px] text-muted-foreground font-mono shrink-0">
                      {current.ismism_code}
                    </span>
                  </div>
                  <div className="flex items-center gap-1.5 mt-0.5">
                    <span className="text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded">
                      {current.field || "未知领域"}
                    </span>
                    {current.tags?.slice(0, 2).map((tag) => (
                      <span
                        key={tag}
                        className="text-[9px] bg-muted text-muted-foreground px-1.5 py-0.5 rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
              {current.self_declare && (
                <p className="text-[11px] text-muted-foreground mt-2 line-clamp-2 italic leading-relaxed">
                  「{current.self_declare.slice(0, 80)}{current.self_declare.length > 80 ? "..." : ""}」
                </p>
              )}
            </div>
          )}
        </div>

        <div className="flex items-center justify-center gap-1.5 mt-3">
          {souls.slice(0, Math.min(souls.length, 12)).map((_, i) => (
            <button
              key={i}
              className={`h-1.5 rounded-full transition-all duration-300 ${
                i === currentIndex
                  ? "w-4 bg-primary"
                  : "w-1.5 bg-muted-foreground/30 hover:bg-muted-foreground/50"
              }`}
              onClick={() => {
                if (timerRef.current) clearInterval(timerRef.current);
                setVisible(false);
                timeoutRef.current = setTimeout(() => {
                  if (!mountedRef.current) return;
                  setCurrentIndex(i);
                  setVisible(true);
                }, 200);
              }}
            />
          ))}
        </div>

        <p className="text-center text-[10px] text-muted-foreground mt-2 animate-pulse">
          匹配中...
        </p>
      </div>
    </div>
  );
}
