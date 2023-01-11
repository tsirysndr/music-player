import React from "react";
import "./index.css";
import App from "./App";
import reportWebVitals from "./reportWebVitals";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";
import { BaseProvider } from "baseui";
import { PLACEMENT, SnackbarProvider } from "baseui/snackbar";
import { theme } from "./Theme";
import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
  split,
  ApolloProvider,
  ApolloLink,
} from "@apollo/client";
import { WebSocketLink } from "@apollo/client/link/ws";
import { render } from "react-dom";
import { getMainDefinition } from "@apollo/client/utilities";
import { SubscriptionClient } from "subscriptions-transport-ws";
import { createTauriLink } from "./TauriLink";

let link: ApolloLink;

if (process.env.REACT_APP_NATIVE_WRAPPER === "tauri") {
  link = createTauriLink();
} else {
  const uri =
    process.env.NODE_ENV === "development"
      ? process.env.REACT_APP_API_URL || "http://localhost:3001/graphql"
      : // eslint-disable-next-line no-restricted-globals
        `${origin}/graphql`;

  const httpLink = createHttpLink({
    uri,
  });

  const wsLink = new WebSocketLink(
    new SubscriptionClient(uri.replace("http", "ws"))
  );

  const splitLink = split(
    ({ query }) => {
      const definition = getMainDefinition(query);
      return (
        definition.kind === "OperationDefinition" &&
        definition.operation === "subscription"
      );
    },
    wsLink,
    httpLink
  );
  link = splitLink;
}

const client = new ApolloClient({
  link,
  cache: new InMemoryCache(),
});

const engine = new Styletron();

const root = document.getElementById("root") as HTMLElement;

render(
  <React.StrictMode>
    <ApolloProvider client={client}>
      <StyletronProvider value={engine}>
        <BaseProvider theme={theme}>
          <SnackbarProvider placement={PLACEMENT.bottom}>
            <App />
          </SnackbarProvider>
        </BaseProvider>
      </StyletronProvider>
    </ApolloProvider>
  </React.StrictMode>,
  root
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
