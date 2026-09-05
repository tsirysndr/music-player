import React from "react";
import "./index.css";
import App from "./App";
import reportWebVitals from "./reportWebVitals";
import { Provider as StyletronProvider } from "styletron-react";
import { Client as Styletron } from "styletron-engine-atomic";
import { PLACEMENT, SnackbarProvider } from "baseui/snackbar";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Provider as JotaiProvider } from "jotai";
import { createRoot } from "react-dom/client";
import Providers from "./Providers";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

const engine = new Styletron();

const container = document.getElementById("root") as HTMLElement;
const root = createRoot(container);

root.render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <JotaiProvider>
        <StyletronProvider value={engine}>
          <Providers>
            <SnackbarProvider placement={PLACEMENT.bottom}>
              <App />
            </SnackbarProvider>
          </Providers>
        </StyletronProvider>
      </JotaiProvider>
    </QueryClientProvider>
  </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
