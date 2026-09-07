import { PLACEMENT, SnackbarProvider } from "baseui/snackbar";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { Client as Styletron } from "styletron-engine-atomic";
import { Provider as StyletronProvider } from "styletron-react";
import type { Preview } from "@storybook/react-vite";
import Providers from "../src/Providers";
import "../src/index.css";

const engine = new Styletron();

/**
 * Every story gets the providers the app mounts at its root: a router (many
 * components render `<Link>`), Styletron and Base Web, and the app's own
 * theme/query providers.
 *
 * The Apollo `MockedProvider` that used to wrap these went when Apollo did —
 * data now comes from TanStack Query, which `Providers` already sets up.
 */
const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/,
      },
    },
  },
  decorators: [
    (Story) => (
      <MemoryRouter>
        <Routes>
          <Route path="/*" element={<Story />} />
        </Routes>
      </MemoryRouter>
    ),
    (Story) => (
      <StyletronProvider value={engine}>
        <Providers>
          <SnackbarProvider placement={PLACEMENT.bottom}>
            <Story />
          </SnackbarProvider>
        </Providers>
      </StyletronProvider>
    ),
  ],
};

export default preview;
