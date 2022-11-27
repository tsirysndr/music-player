import { ApolloLink, Observable, defaultPrinter, split, Operation } from "@apollo/client";
import { getMainDefinition } from "@apollo/client/utilities";
import { print } from "graphql";

function makeRequest(operation: Operation): Record<string, unknown> {
  return {
    query: defaultPrinter(operation.query, print),
    operationName: operation.operationName,
    variables: operation.variables,
    extensions: operation.extensions,
  };
}

function createTauriSubscriptionsLink(): ApolloLink {
  return new ApolloLink(operation => {
    return new Observable(subscriber => {
      const unsubscribeCallbacks: VoidFunction[] = [];
      async function executeSubscriptionQuery() {
        const { invoke, event: { listen, emit } } = window.__TAURI__;
        let token: any;
        try {
          token = await invoke("execute_graphql_subscription", {
            request: makeRequest(operation)
          });
        } catch (e) {
          subscriber.error(e);
        }
        if (token) {
          const unsubscribe = () => {
            emit(`unsubscribe/${token}`);
          }
          if (subscriber.closed) {
            unsubscribe();
            return;
          }
          unsubscribeCallbacks.push(unsubscribe);

          const unlisten = await listen(`subscriptions/${token}`, (event: any) => {
            subscriber.next(event.payload);
          });
          if (subscriber.closed) {
            unlisten();
            return;
          }
          unsubscribeCallbacks.push(unlisten);
        }
      }
      executeSubscriptionQuery();
      return () => {
        for (const callback of unsubscribeCallbacks) {
          callback();
        }
      };
    });
  });
}

function createTauriBasicLink(): ApolloLink {
  return new ApolloLink(operation => {
    return new Observable(subscriber => {
      async function executeQuery() {
        const { invoke } = window.__TAURI__;
        let result: any;
        try {
          result = await invoke("execute_graphql", {
            request: makeRequest(operation)
          });
        } catch (e) {
          subscriber.error(e);
        }
        if (result) {
          subscriber.next(result);
          subscriber.complete();  
        }
      }
      executeQuery();
    });
  });
}

export function createTauriLink(): ApolloLink {
  return split(
    ({ query }) => {
      const definition = getMainDefinition(query);
      return (
        definition.kind === "OperationDefinition" &&
        definition.operation === "subscription"
      );
    },
    createTauriSubscriptionsLink(),
    createTauriBasicLink(),
  );
}
