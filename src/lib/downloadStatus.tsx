import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { DownloadProgress } from "./types";

export type ConsoleLevel = "info" | "progress" | "warn" | "error" | "game" | "server";

export type ConsoleLine = {
  text: string;
  level: ConsoleLevel;
  ts: string;
};

export type ConsoleFilter = "all" | ConsoleLevel;

type DownloadStatusValue = {
  progress: DownloadProgress | null;
  consoleLines: ConsoleLine[];
  consoleOpen: boolean;
  setConsoleOpen: (open: boolean) => void;
  consoleDetached: boolean;
  consoleFilter: ConsoleFilter;
  setConsoleFilter: (f: ConsoleFilter) => void;
  consoleQuery: string;
  setConsoleQuery: (q: string) => void;
  clearConsole: () => void;
  appendConsole: (text: string, level?: ConsoleLevel) => void;
  openConsoleWindow: () => Promise<void>;
  dockConsole: () => Promise<void>;
};

const DownloadStatusContext = createContext<DownloadStatusValue | null>(null);

function normalizeLevel(raw: string | undefined): ConsoleLevel {
  switch ((raw || "info").toLowerCase()) {
    case "progress":
      return "progress";
    case "warn":
    case "warning":
      return "warn";
    case "error":
      return "error";
    case "game":
      return "game";
    case "server":
      return "server";
    default:
      return "info";
  }
}

export function isConsoleWindow(): boolean {
  try {
    return new URLSearchParams(window.location.search).get("eumlWindow") === "console";
  } catch {
    return false;
  }
}

export function DownloadStatusProvider({ children }: { children: ReactNode }) {
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [consoleLines, setConsoleLines] = useState<ConsoleLine[]>([]);
  const [consoleOpen, setConsoleOpen] = useState(false);
  const [consoleDetached, setConsoleDetached] = useState(false);
  const [consoleFilter, setConsoleFilter] = useState<ConsoleFilter>("all");
  const [consoleQuery, setConsoleQuery] = useState("");

  const appendLocal = useCallback((line: ConsoleLine) => {
    setConsoleLines((prev) => {
      const next = [...prev, line];
      return next.length > 2000 ? next.slice(-2000) : next;
    });
  }, []);

  const appendConsole = useCallback(
    (text: string, level: ConsoleLevel = "info") => {
      void invoke("append_console", { text, level }).catch(() => {
        appendLocal({
          text,
          level,
          ts: new Date().toLocaleTimeString(),
        });
      });
    },
    [appendLocal],
  );

  useEffect(() => {
    let cancelled = false;
    invoke<ConsoleLine[]>("get_console_lines")
      .then((lines) => {
        if (!cancelled && Array.isArray(lines)) {
          setConsoleLines(
            lines.map((l) => ({
              text: l.text,
              level: normalizeLevel(l.level),
              ts: l.ts || new Date().toLocaleTimeString(),
            })),
          );
        }
      })
      .catch(() => undefined);

    const unlisteners: Array<() => void> = [];
    listen<ConsoleLine>("euml:console-line", (event) => {
      const p = event.payload;
      appendLocal({
        text: p.text,
        level: normalizeLevel(p.level),
        ts: p.ts || new Date().toLocaleTimeString(),
      });
    }).then((fn) => unlisteners.push(fn));

    listen("euml:console-cleared", () => {
      setConsoleLines([]);
    }).then((fn) => unlisteners.push(fn));

    listen<DownloadProgress>("euml:download-progress", (event) => {
      setProgress(event.payload);
    }).then((fn) => unlisteners.push(fn));

    // Track whether the detached console window exists
    const poll = window.setInterval(() => {
      WebviewWindow.getByLabel("console")
        .then((w) => setConsoleDetached(Boolean(w)))
        .catch(() => setConsoleDetached(false));
    }, 1500);

    return () => {
      cancelled = true;
      clearInterval(poll);
      unlisteners.forEach((fn) => fn());
    };
  }, [appendLocal]);

  const clearConsole = useCallback(() => {
    void invoke("clear_console").catch(() => setConsoleLines([]));
  }, []);

  const openConsoleWindow = useCallback(async () => {
    try {
      await invoke("open_console_window");
      setConsoleDetached(true);
      setConsoleOpen(false);
    } catch (e) {
      // Fallback to JS API if Rust command fails
      try {
        const existing = await WebviewWindow.getByLabel("console");
        if (existing) {
          await existing.setFocus();
          setConsoleDetached(true);
          setConsoleOpen(false);
          return;
        }
        const url = import.meta.env.DEV
          ? `${window.location.origin}/?eumlWindow=console`
          : "index.html?eumlWindow=console";
        const win = new WebviewWindow("console", {
          url,
          title: "Northstar Console",
          width: 860,
          height: 520,
          minWidth: 480,
          minHeight: 280,
          resizable: true,
          focus: true,
        });
        win.once("tauri://created", () => {
          setConsoleDetached(true);
          setConsoleOpen(false);
        });
        win.once("tauri://error", () => {
          setConsoleDetached(false);
          setConsoleOpen(true);
          appendLocal({
            text: `Console window error: ${String(e)}`,
            level: "error",
            ts: new Date().toLocaleTimeString(),
          });
        });
      } catch (e2) {
        setConsoleOpen(true);
        appendLocal({
          text: `Could not open console window: ${String(e2)}`,
          level: "error",
          ts: new Date().toLocaleTimeString(),
        });
      }
    }
  }, [appendLocal]);

  const dockConsole = useCallback(async () => {
    try {
      await invoke("close_console_window");
    } catch {
      const existing = await WebviewWindow.getByLabel("console");
      if (existing) await existing.close();
    }
    setConsoleDetached(false);
    setConsoleOpen(true);
    if (isConsoleWindow()) {
      try {
        await getCurrentWindow().close();
      } catch {
        /* ignore */
      }
    }
  }, []);

  const value = useMemo<DownloadStatusValue>(
    () => ({
      progress,
      consoleLines,
      consoleOpen,
      setConsoleOpen,
      consoleDetached,
      consoleFilter,
      setConsoleFilter,
      consoleQuery,
      setConsoleQuery,
      clearConsole,
      appendConsole,
      openConsoleWindow,
      dockConsole,
    }),
    [
      progress,
      consoleLines,
      consoleOpen,
      consoleDetached,
      consoleFilter,
      consoleQuery,
      clearConsole,
      appendConsole,
      openConsoleWindow,
      dockConsole,
    ],
  );

  return <DownloadStatusContext.Provider value={value}>{children}</DownloadStatusContext.Provider>;
}

export function useDownloadStatus() {
  const ctx = useContext(DownloadStatusContext);
  if (!ctx) throw new Error("useDownloadStatus outside provider");
  return ctx;
}

export function useFilteredConsoleLines() {
  const { consoleLines, consoleFilter, consoleQuery } = useDownloadStatus();
  return useMemo(() => {
    const q = consoleQuery.trim().toLowerCase();
    return consoleLines.filter((l) => {
      if (consoleFilter !== "all" && l.level !== consoleFilter) return false;
      if (q && !l.text.toLowerCase().includes(q)) return false;
      return true;
    });
  }, [consoleLines, consoleFilter, consoleQuery]);
}
