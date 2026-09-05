import { useEffect, useRef, useState } from "react";
import { subscribe } from "../Api/subscriptions";

/**
 * React hook wrapper around the GraphQL subscription layer.
 * Returns `{ data }` with the latest subscription payload, mirroring the
 * shape of the old Apollo `useSubscription` hooks.
 *
 * `onData` (optional) is called with each payload; it doesn't need to be
 * memoized by the caller.
 */
export function useGraphQLSubscription<TData = any>(
  query: string,
  variables?: Record<string, any>,
  onData?: (data: TData) => void
) {
  const [data, setData] = useState<TData | undefined>(undefined);
  const onDataRef = useRef(onData);
  onDataRef.current = onData;

  useEffect(() => {
    const unsubscribe = subscribe<TData>(query, variables, (payload) => {
      setData(payload);
      onDataRef.current && onDataRef.current(payload);
    });
    return unsubscribe;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query, JSON.stringify(variables)]);

  return { data };
}
