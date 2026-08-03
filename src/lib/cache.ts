type Entry<T> = { value: T; at: number };

const store = new Map<string, Entry<unknown>>();

export function cacheGet<T>(key: string, ttlMs = 60_000): T | undefined {
  const hit = store.get(key);
  if (!hit) return undefined;
  if (Date.now() - hit.at > ttlMs) {
    store.delete(key);
    return undefined;
  }
  return hit.value as T;
}

export function cacheSet<T>(key: string, value: T): T {
  store.set(key, { value, at: Date.now() });
  return value;
}

export function cacheInvalidate(prefix?: string) {
  if (!prefix) {
    store.clear();
    return;
  }
  for (const key of store.keys()) {
    if (key.startsWith(prefix)) store.delete(key);
  }
}

/** Deduplicate in-flight promises for the same key. */
const inflight = new Map<string, Promise<unknown>>();

export function cached<T>(key: string, ttlMs: number, loader: () => Promise<T>): Promise<T> {
  const existing = cacheGet<T>(key, ttlMs);
  if (existing !== undefined) return Promise.resolve(existing);

  const pending = inflight.get(key) as Promise<T> | undefined;
  if (pending) return pending;

  const next = loader()
    .then((value) => {
      cacheSet(key, value);
      inflight.delete(key);
      return value;
    })
    .catch((err) => {
      inflight.delete(key);
      throw err;
    });
  inflight.set(key, next);
  return next;
}
