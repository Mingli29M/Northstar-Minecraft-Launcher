import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { api } from "./api";
import type { FavoriteEntry, FavoriteKind } from "./types";
import { favoriteId } from "./types";

type FavoritesValue = {
  favorites: FavoriteEntry[];
  ids: Set<string>;
  ready: boolean;
  isFavorite: (id: string) => boolean;
  toggle: (payload: {
    kind: FavoriteKind;
    key: string;
    label: string;
    subtitle?: string | null;
    iconUrl?: string | null;
  }) => Promise<void>;
  favoritesOf: (kind: FavoriteKind) => FavoriteEntry[];
};

const FavoritesContext = createContext<FavoritesValue | null>(null);

export function FavoritesProvider({ children }: { children: ReactNode }) {
  const [favorites, setFavorites] = useState<FavoriteEntry[]>([]);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    api
      .listFavorites()
      .then((list) => {
        if (!cancelled) setFavorites(list);
      })
      .catch(() => {
        if (!cancelled) setFavorites([]);
      })
      .finally(() => {
        if (!cancelled) setReady(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const ids = useMemo(() => new Set(favorites.map((f) => f.id)), [favorites]);

  const isFavorite = useCallback((id: string) => ids.has(id), [ids]);

  const toggle = useCallback(
    async (payload: {
      kind: FavoriteKind;
      key: string;
      label: string;
      subtitle?: string | null;
      iconUrl?: string | null;
    }) => {
      const id = favoriteId(payload.kind, payload.key);
      const list = await api.toggleFavorite({
        id,
        kind: payload.kind,
        label: payload.label,
        subtitle: payload.subtitle,
        iconUrl: payload.iconUrl,
      });
      setFavorites(list);
    },
    [],
  );

  const favoritesOf = useCallback(
    (kind: FavoriteKind) => favorites.filter((f) => f.kind === kind),
    [favorites],
  );

  const value = useMemo(
    () => ({ favorites, ids, ready, isFavorite, toggle, favoritesOf }),
    [favorites, ids, ready, isFavorite, toggle, favoritesOf],
  );

  return <FavoritesContext.Provider value={value}>{children}</FavoritesContext.Provider>;
}

export function useFavorites() {
  const ctx = useContext(FavoritesContext);
  if (!ctx) throw new Error("useFavorites outside provider");
  return ctx;
}
