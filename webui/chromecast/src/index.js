import React from "react";
import ReactDOM from "react-dom/client";
import CastProvider from "./CastProvider";
import CustomReceiver from "./CustomReceiver";
import { Client as Styletron } from "styletron-engine-atomic";
import { Provider as StyletronProvider } from "styletron-react";
import { DarkTheme, BaseProvider } from "baseui";
import "./index.css";
import reportWebVitals from "./reportWebVitals";

const engine = new Styletron();

const root = ReactDOM.createRoot(document.getElementById("root"));
root.render(
  <React.StrictMode>
    <CastProvider>
      <StyletronProvider value={engine}>
        <BaseProvider theme={DarkTheme}>
          <CustomReceiver />
        </BaseProvider>
      </StyletronProvider>
    </CastProvider>
  </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
