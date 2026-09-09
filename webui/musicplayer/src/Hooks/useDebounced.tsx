import { useEffect, useState } from "react";

/**
 * A value that settles after typing stops.
 *
 * Search runs over the network — federated across the connected server *and*
 * the local index — so a request per keystroke means most of them are already
 * stale when they land, and "sabbath" costs seven searches to answer one
 * question.
 */
export function useDebounced<T>(value: T, delay = 250): T {
  const [settled, setSettled] = useState(value);

  useEffect(() => {
    const timer = setTimeout(() => setSettled(value), delay);
    // Each keystroke cancels the last, so only the pause at the end fires.
    return () => clearTimeout(timer);
  }, [value, delay]);

  return settled;
}
