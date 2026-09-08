import { SubscriptionClient } from "subscriptions-transport-ws";
import { getWsUrl } from "./fetcher";

let subscriptionClient: SubscriptionClient | undefined;

const getSubscriptionClient = (): SubscriptionClient => {
  if (!subscriptionClient) {
    subscriptionClient = new SubscriptionClient(getWsUrl(), {
      reconnect: true,
    });
  }
  return subscriptionClient;
};

/**
 * Subscribes to a GraphQL subscription and calls `onData` with each result's
 * `data` payload. Returns an unsubscribe function.
 *
 * One shared `SubscriptionClient` over the daemon's websocket endpoint —
 * subscriptions are long-lived and every caller wants the same socket.
 */
export function subscribe<TData = any>(
  query: string,
  variables: Record<string, any> | undefined,
  onData: (data: TData) => void
): () => void {
  const observable = getSubscriptionClient().request({
    query,
    variables,
  });
  const subscription = observable.subscribe({
    next: (result) => {
      if (result.data) {
        onData(result.data as TData);
      }
    },
    error: (error) => {
      console.error(error);
    },
  });
  return () => subscription.unsubscribe();
}
