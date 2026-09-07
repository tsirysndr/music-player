import type { Preview } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Provider as JotaiProvider } from "jotai";
import { mswLoader } from "msw-storybook-addon/csf3";
import { useEffect } from "react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import Providers from "../src/Providers";
import { SKINS } from "../src/State";
import { handlers } from "../src/test/handlers";
import "../src/index.css";

/**
 * Every story gets the providers the app mounts at its root: a router (many
 * components render `<Link>`), TanStack Query, Jotai, and the skin provider.
 * MSW answers the GraphQL calls with the same handlers the test suite uses, so
 * a data-connected component renders real-shaped data here too.
 *
 * Styletron and Base Web went with the design-system rewrite — everything is
 * Tailwind over the skin tokens now.
 */

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false } },
});

const preview: Preview = {
  loaders: [mswLoader()],
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/,
      },
    },
    msw: handlers,
    // The chrome behind a story should be the skin's window colour, not
    // Storybook's white — half these components are light-on-dark.
    backgrounds: { disable: true },
    layout: "fullscreen",
  },
  globalTypes: {
    skin: {
      description: "Which desktop skin to render in",
      defaultValue: "late-night",
      toolbar: {
        title: "Skin",
        icon: "paintbrush",
        items: SKINS.map((skin) => ({ value: skin.id, title: skin.name })),
        dynamicTitle: true,
      },
    },
  },
  decorators: [
    // The toolbar picker writes the same `data-skin` attribute the app writes,
    // so every story can be checked against all five skins.
    (Story, context) => {
      useEffect(() => {
        document.documentElement.dataset.skin = context.globals.skin;
        // `index.css` pins `body { overflow: hidden }` so the app never
        // double-scrolls. In Storybook that clips anything taller than the
        // canvas, so the preview iframe gets its scroll back.
        document.body.style.overflow = "auto";
      }, [context.globals.skin]);
      return <Story />;
    },
    (Story) => (
      <MemoryRouter>
        <Routes>
          <Route path="/*" element={<Story />} />
        </Routes>
      </MemoryRouter>
    ),
    (Story) => (
      <QueryClientProvider client={queryClient}>
        <JotaiProvider>
          <Providers>
            <Story />
          </Providers>
        </JotaiProvider>
      </QueryClientProvider>
    ),
  ],
};

export default preview;
