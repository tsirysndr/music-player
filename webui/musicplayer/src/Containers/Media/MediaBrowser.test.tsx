import { graphql, HttpResponse } from "msw";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  act, renderWithProviders, screen, waitFor, within,
} from "../../test/render";
import { server } from "../../test/server";
import type { GetConnectedServerQuery } from "../../Hooks/GraphQL";
import type { MediaEntry, MediaTrack } from "./api";
import MediaBrowser from "./MediaBrowser";

afterEach(() => vi.restoreAllMocks());

const handle = (account: string, id: string, kind = "container") =>
  `mp-source:v1?${new URLSearchParams({
    resolver: "emby", account, server: "fixture-server", user: account, kind, id,
  })}`;
const track = (account: string, id: string): MediaTrack => ({
  id: handle(account, id, "item"), uri: handle(account, id, "item"),
  title: id, duration: 123, discNumber: 0, trackNumber: null,
});
const entry = (account: string, id: string, overrides: Partial<MediaEntry> = {}): MediaEntry => ({
  id: handle(account, id, overrides.isContainer === false ? "item" : "container"),
  title: id, itemType: "Folder", mediaType: null, isContainer: true,
  seasonNumber: null, episodeNumber: null, track: null, ...overrides,
});
const connected = (id: string): GetConnectedServerQuery => ({
  connectedServer: {
    id, name: id, kind: "emby", url: "http://fixture.invalid",
    username: id, hasPassword: false, connected: true,
  },
});
const discovery = (account: () => string) => server.use(
  graphql.query("GetConnectedServer", () => HttpResponse.json({ data: connected(account()) })),
  graphql.query("MediaBrowser", () => HttpResponse.json({
    data: { mediaBrowser: { serverId: account(), supported: true } },
  })),
);

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

// Deliver headers, then hold JSON even after abort. This reproduces a result
// racing cancellation; the component must also reject it at the commit boundary.
function heldBody() {
  const body = deferred<unknown>();
  const reading = deferred<void>();
  let signal: AbortSignal | null | undefined;
  return {
    body, reading,
    get signal() { return signal; },
    response(init?: RequestInit) {
      signal = init?.signal;
      return Promise.resolve(Object.assign(new Response("{}"), {
        json: () => { reading.resolve(); return body.promise; },
      }));
    },
  };
}

describe("media libraries", () => {
  it("navigates containers, preserves large labels, pages by nextOffset and queues audio only", async () => {
    const account = "account-a";
    discovery(() => account);
    const library = entry(account, "Video library");
    const series = entry(account, "Original series title", { itemType: "Series" });
    const season = entry(account, "Season as named by server", {
      itemType: "Season", seasonNumber: "18446744073709551615",
      // Even unexpected track metadata cannot turn a container into a leaf.
      mediaType: "Video", track: track(account, "not-a-leaf"),
    });
    const episode = entry(account, "My First Ever Ender Dragon Fight! [Ep.40]", {
      isContainer: false, itemType: "Episode", mediaType: "Video",
      seasonNumber: "2025050473", episodeNumber: "9007199254740993",
      track: track(account, "episode"),
    });
    const photo = entry(account, "Photo", { isContainer: false, itemType: "Photo", mediaType: "Photo" });
    const movie = entry(account, "Movie without album", {
      isContainer: false, itemType: "Movie", mediaType: "Video", track: track(account, "movie"),
    });
    const pages = vi.fn();
    const queued = vi.fn();
    const leafRequests = vi.fn();
    server.use(
      graphql.query("BrowseMedia", ({ variables }) => {
        pages(variables);
        const entries = variables.parent === library.id ? [series]
          : variables.parent === series.id ? [season]
          : variables.parent === season.id ? (variables.offset === 0 ? [episode, photo] : [movie])
          : [library];
        return HttpResponse.json({ data: { browseMedia: {
          serverId: account, entries,
          nextOffset: variables.parent === season.id && variables.offset === 0 ? 100 : null,
        } } });
      }),
      graphql.query("MediaContainerTracks", ({ variables }) => {
        expect(variables).toEqual({ serverId: account, parent: library.id, offset: 0, limit: 0 });
        return HttpResponse.json({ data: { mediaContainerTracks: [episode.track, movie.track] } });
      }),
      graphql.mutation("AddTracks", ({ variables }) => {
        queued(variables);
        return HttpResponse.json({ data: { addTracks: true } });
      }),
      graphql.mutation("PlayNext", ({ variables }) => {
        leafRequests(variables);
        return HttpResponse.json({ errors: [{ message: "Receiver cannot play this source" }] });
      }),
    );
    const { user } = renderWithProviders(<MediaBrowser />);
    await user.click(await screen.findByRole("button", { name: `Add all to queue: ${library.title}` }));
    await screen.findByText("Added 2 tracks to the queue.");
    expect(queued).toHaveBeenCalledExactlyOnceWith({ tracks: [episode.track, movie.track] });
    await user.click(screen.getByRole("button", { name: `Open ${library.title}` }));
    await user.click(await screen.findByRole("button", { name: `Open ${series.title}` }));
    const seasonRow = (await screen.findByRole("button", { name: `Open ${season.title}` })).closest("li")!;
    expect(within(seasonRow).getByText(/Season 18446744073709551615/)).toBeInTheDocument();
    expect(within(seasonRow).queryByRole("button", { name: /^Listen|^Play next/ })).toBeNull();
    await user.click(within(seasonRow).getByRole("button", { name: `Open ${season.title}` }));
    await screen.findByText(episode.title);
    expect(screen.getByText(/Season 2025050473 · Episode 9007199254740993/)).toBeInTheDocument();
    expect(within(screen.getByText(photo.title).closest("li")!).queryByRole("button")).toBeNull();
    expect(within(screen.getByRole("navigation", { name: "Media path" })).getByText(series.title)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Load more" }));
    await screen.findByText(movie.title);
    expect(pages).toHaveBeenLastCalledWith({ serverId: account, parent: season.id, offset: 100, limit: 100 });
    expect(screen.queryByRole("button", { name: "Load more" })).toBeNull();
    await user.click(screen.getByRole("button", { name: `Listen to ${episode.title}` }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Receiver cannot play this source");
    expect(leafRequests).toHaveBeenCalledExactlyOnceWith({ trackId: episode.track!.id });
  });

  it("cancels a waiting batch and rejects partial GraphQL data without any AddTracks", async () => {
    const account = "account-a";
    discovery(() => account);
    const library = entry(account, "Library A");
    const queued = vi.fn();
    server.use(
      graphql.query("BrowseMedia", () => HttpResponse.json({ data: { browseMedia: {
        serverId: account, nextOffset: null, entries: [library],
      } } })),
      graphql.mutation("AddTracks", ({ variables }) => {
        queued(variables);
        return HttpResponse.json({ data: { addTracks: true } });
      }),
      graphql.query("MediaContainerTracks", () => HttpResponse.json({
        data: { mediaContainerTracks: [track(account, "partial")] },
        errors: [{ message: "Container traversal failed" }],
      })),
    );
    const held = heldBody();
    const original = globalThis.fetch;
    let delayFirst = true;
    vi.spyOn(globalThis, "fetch").mockImplementation((input, init) => {
      if (delayFirst && String(init?.body).includes("query MediaContainerTracks(")) {
        delayFirst = false;
        return held.response(init);
      }
      return original(input, init);
    });
    try {
      const { user } = renderWithProviders(<MediaBrowser />);
      await user.click(await screen.findByRole("button", { name: `Add all to queue: ${library.title}` }));
      await held.reading.promise;
      await user.click(screen.getByRole("button", { name: "Cancel waiting" }));
      expect(held.signal?.aborted).toBe(true);
      await act(async () => {
        held.body.resolve({ data: { mediaContainerTracks: [track(account, "late")] } });
        await held.body.promise;
      });
      expect(queued).not.toHaveBeenCalled();
      expect(screen.getByText("Queue request cancelled.")).toBeInTheDocument();
      await user.click(screen.getByRole("button", { name: `Add all to queue: ${library.title}` }));
      expect(await screen.findByRole("alert")).toHaveTextContent("Container traversal failed");
      expect(queued).not.toHaveBeenCalled();
    } finally {
      held.body.resolve({});
    }
  });

  it("clears the path on account switch and rejects old rows and a ready-to-commit batch", async () => {
    let account = "account-a";
    discovery(() => account);
    const libraryA = entry(account, "A library");
    const seriesA = entry(account, "A series", { itemType: "Series" });
    const libraryB = entry("account-b", "B library");
    const queued = vi.fn();
    server.use(
      graphql.query("BrowseMedia", ({ variables }) => HttpResponse.json({ data: { browseMedia: {
        serverId: variables.serverId,
        nextOffset: variables.parent === libraryA.id ? 100 : null,
        entries: variables.serverId === "account-b" ? [libraryB]
          : variables.parent === libraryA.id ? [seriesA] : [libraryA],
      } } })),
      graphql.mutation("AddTracks", ({ variables }) => {
        queued(variables);
        return HttpResponse.json({ data: { addTracks: true } });
      }),
    );
    const batch = heldBody();
    const page = heldBody();
    const original = globalThis.fetch;
    vi.spyOn(globalThis, "fetch").mockImplementation((input, init) => {
      const body = JSON.parse(String(init?.body ?? "{}"));
      if (body.query?.includes("query MediaContainerTracks(")) return batch.response(init);
      if (body.query?.includes("query BrowseMedia(") && body.variables.offset === 100) return page.response(init);
      return original(input, init);
    });
    try {
      const { user, queryClient } = renderWithProviders(<MediaBrowser />);
      await user.click(await screen.findByRole("button", { name: `Open ${libraryA.title}` }));
      await user.click(await screen.findByRole("button", { name: `Add all to queue: ${seriesA.title}` }));
      await batch.reading.promise;
      await user.click(screen.getByRole("button", { name: "Load more" }));
      await page.reading.promise;
      await act(async () => {
        account = "account-b";
        queryClient.setQueryData(["GetConnectedServer"], connected(account));
        // Resolve in the same turn as the switch, before React can unmount A.
        batch.body.resolve({ data: { mediaContainerTracks: [track("account-a", "late")] } });
        page.body.resolve({ data: { browseMedia: { serverId: "account-a", nextOffset: null,
          entries: [entry("account-a", "Late A row")],
        } } });
        await Promise.all([batch.body.promise, page.body.promise]);
      });
      await screen.findByRole("button", { name: `Open ${libraryB.title}` });
      await waitFor(() => expect(batch.signal?.aborted).toBe(true));
      expect(queued).not.toHaveBeenCalled();
      expect(screen.queryByText("Late A row")).toBeNull();
      expect(screen.queryByText(seriesA.title)).toBeNull();
      const path = screen.getByRole("navigation", { name: "Media path" });
      expect(within(path).queryByText(libraryA.title)).toBeNull();
      expect(within(path).getByText("Libraries")).toHaveAttribute("aria-current", "page");
    } finally {
      batch.body.resolve({});
      page.body.resolve({});
    }
  });
});
