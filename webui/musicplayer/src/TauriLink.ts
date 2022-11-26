import { ApolloLink, Observable, defaultPrinter } from "@apollo/client";
import { print } from "graphql";

export function createTauriLink(): ApolloLink {
  return new ApolloLink(operation => {
    return new Observable(subscriber => {
      const { invoke } = window.__TAURI__;
      invoke('execute_graphql', {
        request: {
          query: defaultPrinter(operation.query, print),
          operationName: operation.operationName,
          variables: operation.variables,
          extensions: operation.extensions,
        },
      }).then((res: any) => {
        if (res.data) {
          subscriber.next(res);
          subscriber.complete();
        } else {
          subscriber.error(res.errors);
        }
        console.log(operation.operationName, res);
      }).catch((e: unknown) => {
        subscriber.error(e);
      });
    });
  });
}