import React from "react";
import ReactDOM from "react-dom/client";
import "./index.css";
import App from "./App";
import reportWebVitals from "./reportWebVitals";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";
import { BaseProvider } from "baseui";
import { theme } from "./Theme";
import { ApolloClient, InMemoryCache, ApolloProvider } from "@apollo/client";

const uri =
  process.env.NODE_ENV === "development"
    ? process.env.REACT_APP_API_URL || "http://localhost:3001/graphql"
    : // eslint-disable-next-line no-restricted-globals
      "/graphql";

const client = new ApolloClient({
  uri,
  cache: new InMemoryCache(),
});

const engine = new Styletron();

const root = ReactDOM.createRoot(
  document.getElementById("root") as HTMLElement
);

root.render(
  <React.StrictMode>
    <ApolloProvider client={client}>
      <StyletronProvider value={engine}>
        <BaseProvider theme={theme}>
          <App />
        </BaseProvider>
      </StyletronProvider>
    </ApolloProvider>
  </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
