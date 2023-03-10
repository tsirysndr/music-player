import { Client as Styletron } from "styletron-engine-atomic";
import { Provider as StyletronProvider } from "styletron-react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { PLACEMENT, SnackbarProvider } from "baseui/snackbar";
import { MockedProvider } from "@apollo/client/testing";
import Providers from "../src/Providers";
import "../src/index.css";

const engine = new Styletron();

export const parameters = {
  actions: { argTypesRegex: "^on[A-Z].*" },
  controls: {
    matchers: {
      color: /(background|color)$/i,
      date: /Date$/,
    },
  },
  reactRouter: {
    routePath: "/albums",
  },
};

const reactRouterDecorator = (Story) => {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/*" element={<Story />} />
      </Routes>
    </MemoryRouter>
  );
};

export const decorators = [
  reactRouterDecorator,
  (Story) => (
    <MockedProvider mocks={[]} addTypename={false}>
      <StyletronProvider value={engine}>
        <Providers>
          <SnackbarProvider placement={PLACEMENT.bottom}>
            <Story />
          </SnackbarProvider>
        </Providers>
      </StyletronProvider>
    </MockedProvider>
  ),
];
