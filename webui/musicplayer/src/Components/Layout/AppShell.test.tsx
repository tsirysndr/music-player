import { QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createStore, Provider as JotaiProvider } from "jotai";
import type { ReactNode } from "react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import Providers from "../../Providers";
import {
  mutedAtom,
  nowPlayingAtom,
  sidebarOpenAtom,
  volumeAtom,
  volumeLoadedAtom,
  type NowPlaying,
} from "../../State";
import {
  makeQueryClient,
  renderWithProviders,
  screen,
  waitFor,
  within,
} from "../../test/render";
import AppShell from "./AppShell";

const setup = (route = "/albums") =>
  renderWithProviders(
    <AppShell>
      <p>page content</p>
    </AppShell>,
    { route }
  );

/**
 * Like `setup`, but with the now-playing atom seeded — `AppStateSync` is what
 * fills it in the running app, and the fullscreen shortcut is gated on it.
 */
const setupPlaying = (
  nowPlaying?: NowPlaying,
  seed?: (store: ReturnType<typeof createStore>) => void
) => {
  const store = createStore();
  if (nowPlaying) store.set(nowPlayingAtom, nowPlaying);
  seed?.(store);

  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={makeQueryClient()}>
      <JotaiProvider store={store}>
        <Providers>
          <MemoryRouter initialEntries={["/albums"]}>
            <Routes>
              <Route path="/*" element={children} />
            </Routes>
          </MemoryRouter>
        </Providers>
      </JotaiProvider>
    </QueryClientProvider>
  );

  return {
    user: userEvent.setup(),
    store,
    ...render(
      <AppShell>
        <p>page content</p>
      </AppShell>,
      { wrapper }
    ),
  };
};

/**
 * The shell with the mixer seeded and its one-time read already marked done,
 * so the fetch cannot land mid-test and overwrite the level under assertion.
 */
const setupVolume = (volume = 0.5, muted = false) =>
  setupPlaying(undefined, (store) => {
    store.set(volumeLoadedAtom, true);
    store.set(volumeAtom, volume);
    store.set(mutedAtom, muted);
  });

const TRACK: NowPlaying = {
  id: "e5f9895f3560dce3181eff25ca349f43",
  title: "End Of The Beginning",
  artist: "Black Sabbath",
  cover: "/covers/3b16b0caed8f12736ae5debf7363fc79.jpg",
  duration: 486_295,
  progress: 0,
  isPlaying: true,
};

describe("AppShell", () => {
  it("renders the page inside the window chrome", () => {
    setup();
    expect(screen.getByText("page content")).toBeInTheDocument();
    expect(screen.getByRole("main")).toBeInTheDocument();
  });

  it("titles the header from the route", () => {
    setup("/albums");
    expect(
      screen.getByRole("heading", { name: "Albums", level: 1 })
    ).toBeInTheDocument();
  });

  it("takes an explicit title over the route's", () => {
    renderWithProviders(
      <AppShell title="13 (Deluxe Version)">
        <p>page content</p>
      </AppShell>,
      { route: "/albums/1" }
    );
    expect(
      screen.getByRole("heading", { name: "13 (Deluxe Version)", level: 1 })
    ).toBeInTheDocument();
  });

  it("lights the section the route belongs to", () => {
    setup("/extensions");
    const sidebar = within(screen.getByRole("complementary"));
    // The sidebar entry is a link; the active one carries the accent rail.
    expect(sidebar.getByRole("link", { name: "Extensions" })).toHaveClass(
      "bg-selected"
    );
    expect(sidebar.getByRole("link", { name: "Albums" })).not.toHaveClass(
      "bg-selected"
    );
  });

  it("lists every section in the sidebar and on the bottom bar", () => {
    setup();
    for (const label of [
      "Albums",
      "Artists",
      "Tracks",
      "Liked",
      "Playlists",
      "Internet Radio",
      "Extensions",
    ]) {
      expect(screen.getAllByRole("link", { name: label }).length).toBeGreaterThan(
        0
      );
    }
  });

  describe("the sidebar toggle", () => {
    it("collapses and restores the sidebar", async () => {
      const { user } = setup();
      const sidebar = screen.getByRole("complementary");
      expect(sidebar).toHaveStyle({ width: "208px" });

      await user.click(
        screen.getByRole("button", { name: "Toggle sidebar" })
      );
      expect(sidebar).toHaveStyle({ width: "0px" });

      await user.click(
        screen.getByRole("button", { name: "Toggle sidebar" })
      );
      expect(sidebar).toHaveStyle({ width: "208px" });
    });

    /** `b`, as on the desktop. */
    it("responds to the keyboard shortcut", async () => {
      const { user } = setup();
      const sidebar = screen.getByRole("complementary");

      await user.keyboard("b");
      expect(sidebar).toHaveStyle({ width: "0px" });
    });
  });

  describe("the queue drawer", () => {
    it("opens from the header and closes from its own button", async () => {
      const { user } = setup();
      expect(screen.queryByRole("heading", { name: "Queue" })).toBeNull();

      await user.click(screen.getByRole("button", { name: "Toggle queue" }));
      expect(
        screen.getByRole("heading", { name: "Queue" })
      ).toBeInTheDocument();

      await user.click(screen.getByRole("button", { name: "Close queue" }));
      expect(screen.queryByRole("heading", { name: "Queue" })).toBeNull();
    });

    /** `q`, as on the desktop. */
    it("responds to the keyboard shortcut", async () => {
      const { user } = setup();
      await user.keyboard("q");
      expect(
        screen.getByRole("heading", { name: "Queue" })
      ).toBeInTheDocument();
    });
  });

  describe("the sidebar status row", () => {
    /**
     * It switches the *source* — where the library is read from. Where the
     * audio comes out is the player bar's picker, and conflating the two is
     * what made this row open the cast dialog.
     */
    it("opens the server switcher rather than the cast picker", async () => {
      const { user } = setupPlaying(TRACK);

      await user.click(
        await screen.findByTitle("Choose which server the library is read from")
      );

      expect(
        await screen.findByPlaceholderText(
          "Search servers, or type an address to connect…"
        )
      ).toBeInTheDocument();
      expect(screen.queryByText("Play to")).toBeNull();
    });

    /** `C` opens it too, matching the desktop and the TUI. */
    it("opens the switcher on C", async () => {
      const { user } = setupPlaying(TRACK);
      await user.keyboard("C");
      expect(
        await screen.findByPlaceholderText(
          "Search servers, or type an address to connect…"
        )
      ).toBeInTheDocument();
    });

    it("lists the saved servers, and the local library", async () => {
      const { user } = setupPlaying(TRACK);
      await user.keyboard("C");
      expect(await screen.findByText("Living room NAS")).toBeInTheDocument();
      expect(screen.getByText("This machine")).toBeInTheDocument();
    });

    /**
     * A search that finds nothing still has somewhere to go: the typed text is
     * offered as an address. A peer daemon is the one kind that needs no
     * credentials, so it is the only thing a bare address can mean.
     */
    it("offers to connect to whatever was typed when nothing matches", async () => {
      const { user } = setupPlaying(TRACK);
      await user.keyboard("C");

      const input = await screen.findByPlaceholderText(
        "Search servers, or type an address to connect…"
      );
      await user.type(input, "studio.lan");

      expect(
        await screen.findByText("Connect to studio.lan")
      ).toBeInTheDocument();
    });

    it("does not offer it while something still matches", async () => {
      const { user } = setupPlaying(TRACK);
      await user.keyboard("C");

      const input = await screen.findByPlaceholderText(
        "Search servers, or type an address to connect…"
      );
      await user.type(input, "Living");

      expect(await screen.findByText("Living room NAS")).toBeInTheDocument();
      expect(screen.queryByText(/^Connect to /)).toBeNull();
    });

    /** Renderers, not the queue: the queue has the right panel. */
    it("keeps the play-to picker on the player bar", async () => {
      const { user } = setupPlaying(TRACK);
      await user.click(await screen.findByRole("button", { name: "Play to" }));
      expect(
        await screen.findByRole("heading", { name: "Play to" })
      ).toBeInTheDocument();
      expect(await screen.findByText("This computer")).toBeInTheDocument();
    });
  });

  describe("the full player and the sidebar", () => {
    /** The canvas wants the window; the sidebar navigates a hidden page. */
    it("hides the sidebar while the full player is open", async () => {
      const { user, store } = setupPlaying(TRACK);
      expect(store.get(sidebarOpenAtom)).toBe(true);

      await user.keyboard("f");
      await waitFor(() => expect(store.get(sidebarOpenAtom)).toBe(false));

      await user.keyboard("{Escape}");
      await waitFor(() => expect(store.get(sidebarOpenAtom)).toBe(true));
    });

    /** Someone who had it collapsed must not get it back. */
    it("restores what it was, not what it assumes", async () => {
      const { user, store } = setupPlaying(TRACK, (store) =>
        store.set(sidebarOpenAtom, false)
      );

      await user.keyboard("f");
      expect(store.get(sidebarOpenAtom)).toBe(false);

      await user.keyboard("{Escape}");
      await waitFor(() => expect(store.get(sidebarOpenAtom)).toBe(false));
    });

    /** Opening it by hand over the canvas is a deliberate override. */
    it("leaves the sidebar alone once it has been reopened by hand", async () => {
      const { user, store } = setupPlaying(TRACK);

      await user.keyboard("f");
      await waitFor(() => expect(store.get(sidebarOpenAtom)).toBe(false));

      await user.keyboard("b");
      await waitFor(() => expect(store.get(sidebarOpenAtom)).toBe(true));

      await user.keyboard("{Escape}");
      await waitFor(() => expect(store.get(sidebarOpenAtom)).toBe(true));
    });
  });

  describe("the volume shortcuts", () => {
    it("raises the volume with + and =", async () => {
      const { user, store } = setupVolume(0.5);
      await user.keyboard("+");
      expect(store.get(volumeAtom)).toBeCloseTo(0.55);
      await user.keyboard("=");
      expect(store.get(volumeAtom)).toBeCloseTo(0.6);
    });

    it("lowers the volume with - and _", async () => {
      const { user, store } = setupVolume(0.5);
      await user.keyboard("-");
      expect(store.get(volumeAtom)).toBeCloseTo(0.45);
      await user.keyboard("_");
      expect(store.get(volumeAtom)).toBeCloseTo(0.4);
    });

    it("clamps at both ends rather than wrapping", async () => {
      const { user, store } = setupVolume(1);
      await user.keyboard("+");
      expect(store.get(volumeAtom)).toBe(1);

      const quiet = setupVolume(0);
      await quiet.user.keyboard("-");
      expect(quiet.store.get(volumeAtom)).toBe(0);
    });

    it("toggles mute with m", async () => {
      const { user, store } = setupVolume(0.5);
      await user.keyboard("m");
      expect(store.get(mutedAtom)).toBe(true);
      await user.keyboard("m");
      expect(store.get(mutedAtom)).toBe(false);
    });

    /** Asking for a level is asking to hear it, so a nudge unmutes. */
    it("unmutes when the volume is nudged, keeping the level", async () => {
      const { user, store } = setupVolume(0.5, true);
      await user.keyboard("+");
      expect(store.get(mutedAtom)).toBe(false);
      expect(store.get(volumeAtom)).toBeCloseTo(0.55);
    });

    /** Typing "-" into a filter must not turn the music down. */
    it("ignores the keys while a field has focus", async () => {
      const { user, store } = setupVolume(0.5);
      const field = document.createElement("input");
      document.body.appendChild(field);
      field.focus();
      await user.keyboard("+m-");
      expect(store.get(volumeAtom)).toBeCloseTo(0.5);
      expect(store.get(mutedAtom)).toBe(false);
      field.remove();
    });
  });

  describe("the fullscreen shortcut", () => {
    /**
     * The full player is a canvas for the current track's art and title, so
     * `f` with nothing playing must not open an empty one.
     */
    it("does nothing while nothing is playing", async () => {
      const { user } = setupPlaying();
      await user.keyboard("f");
      expect(
        screen.queryByRole("button", { name: "Close full player" })
      ).toBeNull();
    });

    /**
     * The canvas is the artwork and a back button — the title and transport
     * stay in the player bar below, which it deliberately stops short of.
     */
    it("opens the full player while a track is playing", async () => {
      const { user } = setupPlaying(TRACK);
      await user.keyboard("f");

      expect(
        await screen.findByRole("button", { name: "Close full player" })
      ).toBeInTheDocument();
      // Exactly one copy of the art: the canvas. The player bar hides its
      // thumbnail while the canvas is up rather than showing it twice.
      expect(screen.getAllByAltText("End Of The Beginning")).toHaveLength(1);
      expect(
        screen.queryByRole("button", { name: "Open full player" })
      ).toBeNull();
    });

    it("toggles back closed on a second press", async () => {
      const { user } = setupPlaying(TRACK);
      await user.keyboard("f");
      await screen.findByRole("button", { name: "Close full player" });

      await user.keyboard("f");
      expect(
        screen.queryByRole("button", { name: "Close full player" })
      ).toBeNull();
    });

    /** A station counts as playing: it has a name before any ICY metadata. */
    it("opens for a radio station", async () => {
      const { user } = setupPlaying({
        id: "radio:nightride-fm",
        title: "Nightride FM",
        duration: 0,
        progress: 0,
        isPlaying: true,
      });

      await user.keyboard("f");
      expect(
        await screen.findByRole("button", { name: "Close full player" })
      ).toBeInTheDocument();
    });

    it("closes on Escape", async () => {
      const { user } = setupPlaying(TRACK);
      await user.keyboard("f");
      await screen.findByRole("button", { name: "Close full player" });

      await user.keyboard("{Escape}");
      expect(
        screen.queryByRole("button", { name: "Close full player" })
      ).toBeNull();
    });
  });

  describe("the skin shortcut", () => {
    it("cycles the skin on `s`", async () => {
      const { user } = setup();
      const before = document.documentElement.dataset.skin;

      await user.keyboard("s");
      await waitFor(() =>
        expect(document.documentElement.dataset.skin).not.toBe(before)
      );
    });
  });

  /**
   * Typing "radio" into a filter must not navigate away on the `r`, so the
   * shortcuts stand down while a text field has focus.
   */
  it("ignores shortcuts while a field has focus", async () => {
    const { user } = renderWithProviders(
      <AppShell>
        <input aria-label="Filter" />
      </AppShell>,
      { route: "/albums" }
    );

    const sidebar = screen.getByRole("complementary");
    await user.click(screen.getByLabelText("Filter"));
    await user.keyboard("b");

    expect(sidebar).toHaveStyle({ width: "208px" });
    expect(screen.getByLabelText("Filter")).toHaveValue("b");
  });

  it("shows a back button only when the page provides one", async () => {
    const { user } = setup();
    expect(screen.queryByRole("button", { name: "Back" })).toBeNull();

    const onBack = vi.fn();
    renderWithProviders(
      <AppShell title="Album" onBack={onBack}>
        <p>page content</p>
      </AppShell>,
      { route: "/albums/1" }
    );

    await user.click(screen.getByRole("button", { name: "Back" }));
    expect(onBack).toHaveBeenCalled();
  });
});
