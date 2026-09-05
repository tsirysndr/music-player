import { SubscriptionClient } from "subscriptions-transport-ws";
import { getWsUrl, isTauri } from "./fetcher";

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
 * Subscribes to a GraphQL subscription and calls `onData` with each
 * result's `data` payload. Returns an unsubscribe function.
 *
 * - In web mode, a single shared SubscriptionClient
 *   (subscriptions-transport-ws) is used over the websocket endpoint.
 * - In tauri mode, the subscription is started through the
 *   `execute_graphql_subscription` tauri command; results arrive on the
 *   `subscriptions/{token}` event, and unsubscribing emits
 *   `unsubscribe/{token}`.
 */
export function subscribe<TData = any>(
  query: string,
  variables: Record<string, any> | undefined,
  onData: (data: TData) => void
): () => void {
  if (isTauri()) {
    let closed = false;
    const unsubscribeCallbacks: VoidFunction[] = [];
    const executeSubscriptionQuery = async () => {
      const {
        core: { invoke },
        event: { listen, emit },
      } = window.__TAURI__;
      let token: any;
      try {
        token = await invoke("execute_graphql_subscription", {
          request: {
            query,
            variables,
          },
        });
      } catch (e) {
        console.error(e);
      }
      if (token) {
        const unsubscribe = () => {
          emit(`unsubscribe/${token}`);
        };
        if (closed) {
          unsubscribe();
          return;
        }
        unsubscribeCallbacks.push(unsubscribe);

        const unlisten = await listen(
          `subscriptions/${token}`,
          (event: any) => {
            if (event.payload?.data) {
              onData(event.payload.data as TData);
            }
          }
        );
        if (closed) {
          unlisten();
          return;
        }
        unsubscribeCallbacks.push(unlisten);
      }
    };
    executeSubscriptionQuery();
    return () => {
      closed = true;
      for (const callback of unsubscribeCallbacks) {
        callback();
      }
    };
  }

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
