import { useCallback, useSyncExternalStore } from "react";
import { useLikeTrackMutation } from "./GraphQL";

// Liked track ids live in localStorage (the daemon has no favorites store);
// every toggle is also forwarded to Rocksky through the likeTrack mutation.
const STORAGE_KEY = "liked-tracks";

const listeners = new Set<() => void>();
let cache: { raw: string | null; ids: Set<string> } = {
  raw: null,
  ids: new Set(),
};

const read = (): Set<string> => {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (raw !== cache.raw) {
    try {
      cache = { raw, ids: new Set(raw ? JSON.parse(raw) : []) };
    } catch {
      cache = { raw, ids: new Set() };
    }
  }
  return cache.ids;
};

const write = (ids: Set<string>) => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(Array.from(ids)));
  listeners.forEach((notify) => notify());
};

const subscribe = (notify: () => void) => {
  listeners.add(notify);
  return () => {
    listeners.delete(notify);
  };
};

export const useLikes = () => {
  const ids = useSyncExternalStore(subscribe, read);
  const { mutate: likeTrack } = useLikeTrackMutation();

  const isLiked = useCallback((id: string) => ids.has(id), [ids]);

  const toggleLike = useCallback(
    (id: string) => {
      const next = new Set(read());
      const like = !next.has(id);
      if (like) {
        next.add(id);
      } else {
        next.delete(id);
      }
      write(next);
      likeTrack({ id, like });
    },
    [likeTrack]
  );

  return { isLiked, toggleLike };
};
