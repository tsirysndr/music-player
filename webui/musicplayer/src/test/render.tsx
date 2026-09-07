import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, type RenderOptions } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Provider as JotaiProvider } from "jotai";
import type { ReactElement, ReactNode } from "react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import Providers from "../Providers";

/**
 * Render helpers.
 *
 * `renderWithProviders` mounts what the app mounts at its root — router,
 * TanStack Query, Jotai, the skin provider — so a component that navigates or
 * fetches behaves in a test the way it does in the app.
 */

/**
 * A fresh client per render, with retries off.
 *
 * Retrying inside a test means a deliberately-failing handler takes three
 * round trips to surface, and a client shared between tests leaks one test's
 * cached data into the next.
 */
export const makeQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0, staleTime: 0 },
      mutations: { retry: false },
    },
  });

export type RenderWithProvidersOptions = Omit<RenderOptions, "wrapper"> & {
  /** Initial history entries, e.g. `["/albums/123"]`. */
  route?: string;
  /** The route pattern to mount under, when the component reads `useParams`. */
  path?: string;
  queryClient?: QueryClient;
};

export function renderWithProviders(
  ui: ReactElement,
  {
    route = "/",
    path = "/*",
    queryClient = makeQueryClient(),
    ...options
  }: RenderWithProvidersOptions = {}
) {
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <JotaiProvider>
        <Providers>
          <MemoryRouter initialEntries={[route]}>
            <Routes>
              <Route path={path} element={children} />
            </Routes>
          </MemoryRouter>
        </Providers>
      </JotaiProvider>
    </QueryClientProvider>
  );

  return {
    // `userEvent` has to be set up before render for its pointer bookkeeping.
    user: userEvent.setup(),
    queryClient,
    ...render(ui, { wrapper: Wrapper, ...options }),
  };
}

/**
 * Render with only the router — for a presentational component that neither
 * fetches nor reads the skin, where the extra providers are noise.
 */
export function renderWithRouter(
  ui: ReactElement,
  { route = "/", path = "/*", ...options }: RenderWithProvidersOptions = {}
) {
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <MemoryRouter initialEntries={[route]}>
      <Routes>
        <Route path={path} element={children} />
      </Routes>
    </MemoryRouter>
  );
  return {
    user: userEvent.setup(),
    ...render(ui, { wrapper: Wrapper, ...options }),
  };
}

export * from "@testing-library/react";
export { userEvent };
