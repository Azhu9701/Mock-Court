"use client";

import { useCallback, useEffect, useRef, useState } from "react";

export interface ProcessStep {
  event: string;
  message: string;
  soulName: string | null;
  timestamp: Date;
}

export interface LogEntry {
  timestamp: Date;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
}

export interface WsEvent {
  event_type: string;
  payload: string;
  soul_name: string | null;
  seq: number;
}

export interface SoulMessage {
  soulName: string;
  content: string;
  isStreaming: boolean;
  error: string | null;
  ismismCode?: string;
}

export interface CostInfo {
  llm_calls: number;
  tokens_used: number;
  estimated_cost: string;
}

export interface CollisionEvent {
  from: string;
  to: string;
  content: string;
  injected?: boolean;
}

const MAX_RETRIES = 3;
const FLUSH_INTERVAL_MS = 50; // batch state updates at ~20fps

const EVENT_LABEL: Record<string, string> = {
  session_started: "SessionStarted",
  entry_classified: "EntryClassified",
  soul_started: "SoulStarted",
  synthesis_started: "SynthesisStarted",
};

export function useWebSocket(sessionId: string) {
  const wsRef = useRef<WebSocket | null>(null);
  const retryRef = useRef(0);
  const bufferRef = useRef<Record<string, SoulMessage>>({});
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingFlushRef = useRef(false);
  const mountedRef = useRef(true);

  // forceTick: counter that React sees — incrementing it triggers a re-render
  // This is the ONLY React state for streaming content, avoiding per-token setState overhead
  const [tick, setTick] = useState(0);
  // Snapshot of buffer for React consumption — only updated on flush
  const [messages, setMessages] = useState<Record<string, SoulMessage>>({});

  const [synthesis, setSynthesis] = useState("");
  const synthesisRef = useRef("");
  const [status, setStatus] = useState<"connecting" | "streaming" | "done" | "error">("connecting");
  const [error, setError] = useState<string | null>(null);
  const [processSteps, setProcessSteps] = useState<ProcessStep[]>([]);
  const [cost, setCost] = useState<CostInfo | null>(null);
  const [collisions, setCollisions] = useState<CollisionEvent[]>([]);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  // Flush buffer to React state at controlled intervals
  const scheduleFlush = useCallback(() => {
    if (pendingFlushRef.current || !mountedRef.current) return;
    pendingFlushRef.current = true;
    flushTimerRef.current = setTimeout(() => {
      pendingFlushRef.current = false;
      if (!mountedRef.current) return;
      // Snapshot buffer content into React state
      setMessages({ ...bufferRef.current });
      setSynthesis(synthesisRef.current);
      setTick((t) => t + 1);
    }, FLUSH_INTERVAL_MS);
  }, []);

  // Immediate flush (for done/error events that shouldn't wait)
  const flushImmediate = useCallback(() => {
    if (flushTimerRef.current) {
      clearTimeout(flushTimerRef.current);
      pendingFlushRef.current = false;
    }
    if (!mountedRef.current) return;
    setMessages({ ...bufferRef.current });
    setSynthesis(synthesisRef.current);
    setTick((t) => t + 1);
  }, []);

  const addStep = (event: string, message: string, soulName: string | null) => {
    const label = EVENT_LABEL[event] || event;
    setProcessSteps((prev) => [...prev, { event: label, message, soulName, timestamp: new Date() }]);
  };

  const addLog = (message: string, type: LogEntry['type'] = 'info') => {
    setLogs((prev) => [...prev, { timestamp: new Date(), message, type }]);
  };

  const connect = useCallback(() => {
    const wsHost = (process.env.NEXT_PUBLIC_API_URL || "http://127.0.0.1:3096").replace("http://", "ws://").replace("/api/v1", "");
    const url = `${wsHost}/ws/possess/${sessionId}/main`;
    const ws = new WebSocket(url);
    wsRef.current = ws;
    mountedRef.current = true;
    setStatus("connecting");
    setProcessSteps([]);
    bufferRef.current = {};
    synthesisRef.current = "";
    setMessages({});
    setSynthesis("");
    setCost(null);
    setCollisions([]);
    setLogs([]);

    ws.onopen = () => {
      retryRef.current = 0;
      setStatus("streaming");
      setError(null);
      addLog("WebSocket 连接已建立", 'success');
    };

    ws.onmessage = (e) => {
      const event: WsEvent = JSON.parse(e.data);

      switch (event.event_type) {
        // Process events — update React state directly (low frequency)
        case "session_started":
          addStep(event.event_type, event.payload, event.soul_name);
          addLog(`会话已启动: ${event.payload}`, 'success');
          break;
        case "entry_classified":
          addStep(event.event_type, event.payload, event.soul_name);
          addLog(`入口分流完成: ${event.payload}`, 'info');
          break;
        case "soul_started":
          addStep(event.event_type, event.payload, event.soul_name);
          addLog(`正在召唤魂: ${event.soul_name || event.payload}`, 'info');
          break;
        case "synthesis_started":
          addStep(event.event_type, event.payload, event.soul_name);
          addLog(`开始综合分析...`, 'info');
          break;

        // Soul streaming — write to ref (synchronous), schedule throttled flush
        case "soul_token": {
          const soulName = event.soul_name;
          if (soulName) {
            if (!bufferRef.current[soulName]) {
              bufferRef.current[soulName] = {
                soulName,
                content: "",
                isStreaming: true,
                error: null,
              };
            } else {
              // Create a new object reference to ensure React detects changes
              bufferRef.current[soulName] = {
                ...bufferRef.current[soulName],
                content: bufferRef.current[soulName].content + event.payload,
              };
            }
            scheduleFlush();
          }
          break;
        }

        case "soul_done":
          if (event.soul_name && bufferRef.current[event.soul_name]) {
            bufferRef.current[event.soul_name] = {
              ...bufferRef.current[event.soul_name],
              isStreaming: false,
            };
            flushImmediate();
          }
          addLog(`魂回应完成: ${event.soul_name}`, 'success');
          break;

        case "soul_error":
          if (event.soul_name && bufferRef.current[event.soul_name]) {
            bufferRef.current[event.soul_name] = {
              ...bufferRef.current[event.soul_name],
              error: event.payload,
              isStreaming: false,
            };
            flushImmediate();
          }
          addLog(`魂出错: ${event.soul_name} - ${event.payload}`, 'error');
          break;

        case "synthesis_chunk":
          synthesisRef.current += event.payload;
          scheduleFlush();
          break;

        case "synthesis_done":
          flushImmediate();
          addLog(`综合分析完成`, 'success');
          break;

        case "collision":
          try {
            const c = JSON.parse(event.payload) as CollisionEvent;
            setCollisions((prev) => [...prev, c]);
          } catch {}
          break;

        case "cost":
          try {
            const c = JSON.parse(event.payload) as CostInfo;
            setCost(c);
          } catch {}
          break;

        case "done":
        case "SessionComplete":
          flushImmediate();
          setStatus("done");
          addLog(`会话完成`, 'success');
          break;
      }
    };

    ws.onclose = () => {
      if (retryRef.current < MAX_RETRIES) {
        setTimeout(connect, Math.pow(2, retryRef.current) * 1000);
        retryRef.current++;
      } else {
        setStatus("error");
        setError("连接已断开，请稍后重试");
      }
    };

    ws.onerror = () => ws.close();
  }, [sessionId, scheduleFlush, flushImmediate]);

  useEffect(() => {
    connect();
    return () => {
      mountedRef.current = false;
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
      wsRef.current?.close();
    };
  }, [connect]);

  return {
    messages,
    synthesis,
    status,
    error,
    processSteps,
    cost,
    collisions,
    logs,
    tick,
    reconnect: connect,
  };
}
