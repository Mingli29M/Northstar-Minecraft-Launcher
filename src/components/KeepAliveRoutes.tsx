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
 * Keeps a bounded set of visited route panes mounted so Astryx trees aren't
 * rebuilt on every nav. Inactive panes use `display: none` (state preserved).
 * Sticky ids (e.g. Launch) are never evicted; others follow LRU up to maxAlive.
 */
export function KeepAliveRoutes({
  panes,
  maxAlive = 3,
  stickyIds = ["launch"],
}: {
  panes: Pane[];
  maxAlive?: number;
  stickyIds?: string[];
}) {
  const { pathname } = useLocation();
  const current = activeId(panes, pathname);
  const [alive, setAlive] = useState<string[]>(() => [current]);
  const [, startTransition] = useTransition();
  const stickyKey = stickyIds.join("\0");

  useEffect(() => {
    startTransition(() => {
      setAlive((prev) => {
        const sticky = new Set(stickyKey.split("\0").filter(Boolean));
        const without = prev.filter((id) => id !== current);
        const next = [...without, current];
        while (next.length > maxAlive) {
          const evictAt = next.findIndex((id) => !sticky.has(id));
          if (evictAt < 0) break;
          next.splice(evictAt, 1);
        }
        return next;
      });
    });
  }, [current, maxAlive, stickyKey]);

  const aliveSet = new Set(alive);

  return (
    <div className="euml-keepalive">
      {panes.map((pane) => {
        if (!aliveSet.has(pane.id) && pane.id !== current) return null;
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
