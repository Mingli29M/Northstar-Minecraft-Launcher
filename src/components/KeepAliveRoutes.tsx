import { type ReactNode, useEffect, useState, useTransition } from "react";
import { useLocation } from "react-router-dom";

type Pane = {
  /** Stable key for keep-alive (not the full URL). */
  id: string;
  match: (pathname: string) => boolean;
  element: ReactNode;
};

function activeId(panes: Pane[], pathname: string): string {
  return panes.find((p) => p.match(pathname))?.id ?? panes[0]?.id ?? "";
}

/**
 * Keeps visited route panes mounted so Astryx trees aren't rebuilt on every nav.
 * Inactive panes use `display: none` (state preserved).
 */
export function KeepAliveRoutes({ panes }: { panes: Pane[] }) {
  const { pathname } = useLocation();
  const current = activeId(panes, pathname);
  const [visited, setVisited] = useState(() => new Set([current]));
  const [, startTransition] = useTransition();

  useEffect(() => {
    startTransition(() => {
      setVisited((prev) => {
        if (prev.has(current)) return prev;
        const next = new Set(prev);
        next.add(current);
        return next;
      });
    });
  }, [current]);

  return (
    <div className="euml-keepalive">
      {panes.map((pane) => {
        if (!visited.has(pane.id) && pane.id !== current) return null;
        const active = pane.id === current;
        return (
          <div
            key={pane.id}
            className={`euml-keepalive-pane${active ? " is-active" : ""}`}
            hidden={!active}
            aria-hidden={!active}
            style={{
              display: active ? "block" : "none",
            }}
          >
            {pane.element}
          </div>
        );
      })}
    </div>
  );
}
