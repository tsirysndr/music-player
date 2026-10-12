import { print, type DocumentNode } from "graphql";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import { getApiUrl } from "../../Api/fetcher";

export const UNCERTAIN = "Request outcome unknown. Refresh to check its state before trying again.";
export const errorMessage = (error: unknown) => error instanceof Error ? error.message : "The source request failed";
class SourceGraphQLError extends Error {}

// No retries, no optional-data fallback, and a finite wait including JSON body
// reads. The linked abort also stops fetch when the component loses ownership.
export async function sourceRequest<T>(document: DocumentNode, variables: object,
  signal: AbortSignal, mutation = false): Promise<T> {
  signal.throwIfAborted();
  const controller = new AbortController();
  const cancel = () => controller.abort();
  signal.addEventListener("abort", cancel, { once: true });
  let timedOut = false;
  const timer = setTimeout(() => { timedOut = true; controller.abort(); }, 35000);
  let abortListener!: () => void;
  const aborted = new Promise<never>((_, reject) => {
    abortListener = () => reject(new Error("Source request cancelled"));
    controller.signal.addEventListener("abort", abortListener, { once: true });
  });
  try {
    const fetchData = async () => {
      const response = await fetch(getApiUrl(), {
        method: "POST", headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query: print(document), variables }), signal: controller.signal,
      });
      if (!response.ok) throw new Error("Source request failed");
      const result: { data?: T; errors?: { message: string }[] } = await response.json();
      controller.signal.throwIfAborted();
      if (result.errors?.length) throw new SourceGraphQLError(result.errors[0].message);
      if (!result.data) throw new Error("Missing source response");
      return result.data;
    };
    return await Promise.race([fetchData(), aborted]);
  } catch (error) {
    if (error instanceof SourceGraphQLError) throw error;
    if (signal.aborted) throw new Error("Source request cancelled");
    const reason = timedOut ? "The source request timed out." : "The source response could not be received.";
    throw new Error(mutation ? reason + " " + UNCERTAIN : reason);
  } finally {
    clearTimeout(timer);
    signal.removeEventListener("abort", cancel);
    controller.signal.removeEventListener("abort", abortListener);
  }
}

export type SourceAttempt = {
  signal: AbortSignal;
  current: () => boolean;
  submit: <T>(send: () => Promise<T>) => Promise<T>;
};

// The ref check and send are synchronous. Cancel/unmount prevents a pending
// multi-page read from submitting; a sent mutation cannot be recalled.
export function useSourceOperation() {
  const active = useRef<{ controller: AbortController; submitted: boolean } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const invalidate = useCallback(() => {
    const old = active.current;
    active.current = null;
    old?.controller.abort();
    return old;
  }, []);
  const cancel = useCallback(() => {
    const old = invalidate();
    if (old) {
      setBusy(false);
      setError("");
      setMessage(old.submitted ? "Stopped waiting. " + UNCERTAIN : "Request cancelled. Nothing was submitted.");
    }
  }, [invalidate]);
  useLayoutEffect(() => () => { invalidate(); }, [invalidate]);

  const run = async <T,>(task: (attempt: SourceAttempt) => Promise<T>,
    onSuccess?: (result: T) => void, successMessage = "") => {
    // Prevent duplicate clicks even before React renders disabled buttons.
    if (active.current) return;
    const owner = { controller: new AbortController(), submitted: false };
    active.current = owner;
    setBusy(true); setError(""); setMessage("");
    const current = () => active.current === owner && !owner.controller.signal.aborted;
    try {
      const result = await task({
        signal: owner.controller.signal, current,
        submit: (send) => {
          if (!current()) return Promise.reject(new Error("Source request cancelled"));
          owner.submitted = true;
          return send();
        },
      });
      if (current()) { setMessage(successMessage); onSuccess?.(result); }
    } catch (cause) {
      if (current()) setError(errorMessage(cause));
    } finally {
      if (active.current === owner) { active.current = null; setBusy(false); }
    }
  };
  return { busy, error, message, run, cancel };
}
