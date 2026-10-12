import { useAtomValue } from "jotai";
import { graphql, HttpResponse } from "msw";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { SourceMediaEntry, SourceMediaTrack } from "../../GraphQL/SourceAccounts/Query";
import { useRescanExtensionsMutation, useSetExtensionEnabledMutation } from "../../Hooks/GraphQL";
import { queueOpenAtom } from "../../State";
import { act, renderWithProviders, screen, waitFor, within } from "../../test/render";
import { server } from "../../test/server";
import { SourceBrowser } from "./Browse";

afterEach(() => vi.restoreAllMocks());

const handle = (account: string, id: string, kind = "item", version = 1) => {
  const fields: Record<string, string> = version === 1 ? {
    resolver: "extension:fixture", account, scope: "local", revision: "1", kind, id,
  } : { resolver: "extension:fixture", account, server: "remote", user: account, kind, id };
  return (version === 1 ? "mp-source:v2?" : "mp-source:v1?") + new URLSearchParams(fields).toString();
};
const track = (account: string, id: string, version = 1): SourceMediaTrack => ({
  id: handle(account, id, "item", version), uri: handle(account, id, "item", version),
  title: id, artist: "Original artist", duration: 12.5, discNumber: 2, trackNumber: 0,
  album: { title: "Original album", artist: "Album artist", year: 2025 },
});
const entry = (account: string, id: string, version = 1, overrides: Partial<SourceMediaEntry> = {}): SourceMediaEntry => ({
  id: handle(account, id, "item", version), title: id, isContainer: false,
  itemType: "Audio", mediaType: "Audio", seasonNumber: null, episodeNumber: null,
  track: track(account, id, version), ...overrides,
});
const body = (accountId: string, entries: SourceMediaEntry[], nextOffset: string | null = null) => ({
  data: { browseSourceMedia: { accountId, entries, nextOffset } },
});
function accounts(version = 1) {
  server.use(
    graphql.query("ExtensionSourceDefinitions", () => HttpResponse.json({ data: { extensionSourceDefinitions: [{
      extensionId: "fixture", name: "Fixture", enabled: true, sourceApiVersion: version, configKeys: [],
    }] } })),
    graphql.query("SourceAccount", ({ variables }) => HttpResponse.json({ data: { sourceAccount: {
      id: variables.id, extensionId: "fixture", name: "Account " + variables.id, sourceApiVersion: version,
      configRevision: "1", identityBound: true, overriddenConfigKeys: [],
    } } })),
  );
}
function QueueState() {
  const open = useAtomValue(queueOpenAtom);
  return <output aria-label="Queue visibility">{open ? "open" : "closed"}</output>;
}
function ExtensionControls() {
  // Use the real generated keys and mutation functions, as ExtensionsWithData
  // does; a test-only mutationKey would not check the cross-page wiring.
  const disable = useSetExtensionEnabledMutation();
  const rescan = useRescanExtensionsMutation();
  return <>
    <button onClick={() => disable.mutate({ id: "fixture", enabled: false })}>Disable extension</button>
    <button onClick={() => rescan.mutate({})}>Rescan extensions</button>
  </>;
}
function heldBody() {
  let release!: (body: unknown) => void;
  let reading!: () => void;
  const body = new Promise<unknown>((resolve) => { release = resolve; });
  const started = new Promise<void>((resolve) => { reading = resolve; });
  let signal: AbortSignal | null | undefined;
  return {
    release, started, get signal() { return signal; },
    response(init?: RequestInit) {
      signal = init?.signal;
      // Deliberately ignore abort while reading, so ownership must also guard
      // the eventual completion. This never starts an actual HTTP server.
      return Promise.resolve(Object.assign(new Response("{}"), { json: () => { reading(); return body; } }));
    },
  };
}

function v1Browser(songCount = 1) {
  accounts();
  const calls = vi.fn();
  const queued = vi.fn();
  const forbidden = vi.fn();
  const played = vi.fn();
  const container = entry("a", "Original collection", 1, {
    id: handle("a", "collection", "container"), isContainer: true,
    // Even a malformed response carrying track tags cannot make a container playable.
    track: track("a", "container-is-not-a-song"),
  });
  const songs = Array.from({ length: songCount }, (_, i) => entry("a", "Song " + i));
  server.use(
    graphql.query("BrowseSourceMedia", ({ variables }) => {
      calls(variables);
      return HttpResponse.json(variables.list !== "TRACKS" ? body("a", [container])
        : body("a", variables.offset === "0" ? songs : [], variables.offset === "0" ? "100" : null));
    }),
    graphql.mutation("AddSourceTracks", ({ variables }) => {
      queued(variables);
      return HttpResponse.json({ data: { addTracks: true } });
    }),
    graphql.query("MediaAudioOptions", () => { forbidden(); return HttpResponse.json({ errors: [{ message: "V1 has no options" }] }); }),
    graphql.mutation("PlayTrack", () => { forbidden(); return HttpResponse.json({ errors: [{ message: "No ID-only lookup" }] }); }),
    graphql.mutation("PlaySelectedSourceMedia", ({ variables }) => {
      played(variables); return HttpResponse.json({ data: { playSelectedMedia: "new-occurrence" } });
    }),
  );
  const view = renderWithProviders(<><SourceBrowser accountId="a" /><QueueState /></>);
  return { ...view, calls, queued, forbidden, played, container, songs };
}

describe("extension source browsing", () => {
  it("navigates all four v1 lists without treating containers as playable tracks", async () => {
    const { user, calls, container, forbidden, queued, played } = v1Browser();
    await screen.findByText("Song 0");
    expect(screen.queryByRole("button", { name: /^Audio options:/ })).toBeNull();
    for (const list of ["Albums", "Artists", "Playlists"]) {
      await user.click(screen.getByRole("button", { name: list }));
      await screen.findByRole("button", { name: "Open tracks in Original collection" });
      expect(calls).toHaveBeenLastCalledWith({ accountId: "a", parent: null, list: list.toUpperCase(), query: null, offset: "0", limit: 100 });
      expect(within(screen.getByText("Original collection").closest("li")!).queryByRole("button", { name: /^Add to queue:/ })).toBeNull();
    }
    await user.click(screen.getByRole("button", { name: "Open tracks in Original collection" }));
    await screen.findByText("Song 0");
    expect(calls).toHaveBeenLastCalledWith({ accountId: "a", parent: container.id, list: "TRACKS", query: null, offset: "0", limit: 100 });
    expect(queued).not.toHaveBeenCalled();
    expect(played).not.toHaveBeenCalled();
    expect(forbidden).not.toHaveBeenCalled();
  });

  it("reads a full v1 page then an empty page and queues complete browse tags once", async () => {
    const { user, calls, queued, songs, forbidden } = v1Browser(100);
    await screen.findByText("Song 0");
    await user.click(screen.getByRole("button", { name: "Add all audio in this view" }));
    await screen.findByText("Finished adding audio from this view.");
    expect(calls).toHaveBeenLastCalledWith({ accountId: "a", parent: null, list: "TRACKS", query: null, offset: "100", limit: 100 });
    expect(queued).toHaveBeenCalledTimes(1);
    expect(queued.mock.calls[0][0].tracks).toHaveLength(100);
    expect(queued.mock.calls[0][0].tracks[0]).toEqual({
      id: songs[0].track!.id, uri: songs[0].track!.uri, title: "Song 0", artist: "Original artist",
      duration: 12.5, discNumber: 2, trackNumber: 0,
      albumTitle: "Original album", albumArtist: "Album artist", albumYear: 2025,
    });
    expect(screen.getByLabelText("Queue visibility")).toHaveTextContent("closed");
    expect(forbidden).not.toHaveBeenCalled();
  });

  it("submits complete v1 browse tags for Listen and Play next without an ID-only fallback", async () => {
    const { user, played, queued, forbidden, songs } = v1Browser();
    await screen.findByText("Song 0");
    await user.click(screen.getByRole("button", { name: "Listen: Song 0" }));
    await screen.findByText("Playback requested.");
    await user.click(screen.getByRole("button", { name: "Play next: Song 0" }));
    await screen.findByText("Track added to play next.");
    const selected = { track: {
      id: songs[0].track!.id, uri: songs[0].track!.uri, title: "Song 0", artist: "Original artist",
      duration: 12.5, discNumber: 2, trackNumber: 0,
      albumTitle: "Original album", albumArtist: "Album artist", albumYear: 2025,
    } };
    expect(played.mock.calls.map(([variables]) => variables)).toEqual([
      { entry: selected, next: false }, { entry: selected, next: true },
    ]);
    expect(screen.getByLabelText("Queue visibility")).toHaveTextContent("closed");
    expect(queued).not.toHaveBeenCalled();
    expect(forbidden).not.toHaveBeenCalled();
  });

  it("navigates and searches v2 with exact offsets and labels, preserving explicit audio index zero", async () => {
    accounts(2);
    const library = entry("a", "Original library", 2, { id: handle("a", "root", "container", 2), isContainer: true, track: null });
    const episode = entry("a", "Original episode title", 2, { seasonNumber: "18446744073709551615", episodeNumber: "9007199254740993" });
    const photo = entry("a", "Photo", 2, { track: null, itemType: "Photo", mediaType: "Photo" });
    const extra = entry("a", "Next page", 2);
    const pages = vi.fn();
    const selected = vi.fn();
    const played = vi.fn();
    server.use(
      graphql.query("BrowseSourceMedia", ({ variables }) => {
        pages(variables);
        return HttpResponse.json(!variables.parent ? body("a", [library])
          : variables.query ? body("a", [episode])
          : variables.offset === "0" ? body("a", [episode, photo], "9007199254740993") : body("a", [extra]));
      }),
      graphql.query("MediaAudioOptions", ({ variables }) => HttpResponse.json({ data: { mediaAudioOptions: {
        source: variables.source, versions: [{ id: "version", name: "Original version", runtimeTicks: "18446744073709551615",
          defaultAudioStreamIndex: 0, unavailableReason: null, audioStreams: [{
            index: 0, title: "Commentary", displayTitle: "Original audio", language: "", codec: "mp3",
            channels: 2, sampleRate: 44100, isDefault: true, unavailableReason: null,
          }, { index: 1, title: "Unavailable", displayTitle: null, language: "en", codec: null,
            channels: null, sampleRate: null, isDefault: false, unavailableReason: "Live source unavailable" }],
        }],
      } } })),
      graphql.mutation("AddSelectedSourceMedia", ({ variables }) => {
        selected(variables); return HttpResponse.json({ data: { addSelectedMedia: ["occurrence-1"] } });
      }),
      graphql.mutation("PlaySelectedSourceMedia", ({ variables }) => {
        played(variables); return HttpResponse.json({ data: { playSelectedMedia: "occurrence-2" } });
      }),
    );
    const { user } = renderWithProviders(<SourceBrowser accountId="a" />);
    await user.click(await screen.findByRole("button", { name: "Open Original library" }));
    await screen.findByText(/Season 18446744073709551615 · Episode 9007199254740993/);
    expect(within(screen.getByText("Photo").closest("li")!).queryByRole("button")).toBeNull();
    await user.click(screen.getByRole("button", { name: "Load more" }));
    await screen.findByText("Next page");
    expect(pages).toHaveBeenLastCalledWith({ accountId: "a", parent: library.id, list: null, query: null, offset: "9007199254740993", limit: 100 });
    await user.type(screen.getByLabelText("Search this source view"), "original title");
    await user.click(screen.getByRole("button", { name: "Search" }));
    await waitFor(() => expect(pages).toHaveBeenLastCalledWith({ accountId: "a", parent: library.id, list: null, query: "original title", offset: "0", limit: 100 }));
    await user.click(await screen.findByRole("button", { name: "Audio options: Original episode title" }));
    await screen.findByRole("option", { name: /Original audio.*Default/ });
    expect(screen.getByRole("option", { name: /Live source unavailable/ })).toBeDisabled();
    await user.selectOptions(screen.getByLabelText("Media version and audio track"), JSON.stringify(["version", 0]));
    await user.click(screen.getByRole("button", { name: "Add with this audio" }));
    await screen.findByText(/Track added/);
    expect(selected).toHaveBeenCalledExactlyOnceWith({ entries: [{
      track: { id: episode.track!.id, uri: episode.track!.uri, title: episode.title, artist: "Original artist",
        duration: 12.5, discNumber: 2, trackNumber: 0, albumTitle: "Original album", albumArtist: "Album artist", albumYear: 2025 },
      choice: { mediaSourceId: "version", audioStreamIndex: 0 },
    }] });
    for (const action of ["listen", "next"]) {
      await user.selectOptions(screen.getByLabelText("With selected audio"), action);
      await user.click(screen.getByRole("button", { name: action === "listen" ? "Listen with this audio" : "Play next with this audio" }));
      await screen.findByText(action === "listen" ? "Playback requested." : "Track added to play next.");
    }
    expect(played.mock.calls.map(([variables]) => variables)).toEqual([
      { entry: selected.mock.calls[0][0].entries[0], next: false },
      { entry: selected.mock.calls[0][0].entries[0], next: true },
    ]);
  });

  it.each(["cancel", "switch account"] as const)("ignores a submitted Listen response after %s without a second command", async (action) => {
    accounts();
    const forbidden = vi.fn();
    server.use(
      graphql.query("BrowseSourceMedia", ({ variables }) => HttpResponse.json(body(String(variables.accountId),
        [entry(String(variables.accountId), "Song " + variables.accountId)]))),
      graphql.mutation(/^(AddSourceTracks|PlayTrack|PlayTrackAt|PlayNext)$/, () => {
        forbidden(); return HttpResponse.json({ errors: [{ message: "Unexpected fallback" }] });
      }),
    );
    const held = heldBody();
    const sent = vi.fn();
    const original = globalThis.fetch;
    vi.spyOn(globalThis, "fetch").mockImplementation((input, init) => {
      const request = JSON.parse(String(init?.body ?? "{}"));
      if (request.query?.includes("mutation PlaySelectedSourceMedia(")) {
        sent(request.variables); return held.response(init);
      }
      return original(input, init);
    });
    const { user, rerender, queryClient } = renderWithProviders(<SourceBrowser accountId="a" />);
    try {
      await screen.findByText("Song a");
      const invalidated = vi.spyOn(queryClient, "invalidateQueries");
      await user.click(screen.getByRole("button", { name: "Listen: Song a" }));
      await waitFor(() => expect(sent).toHaveBeenCalledTimes(1));
      if (action === "cancel") {
        await user.click(screen.getByRole("button", { name: "Cancel waiting" }));
        expect(screen.getByText(/Request outcome unknown/)).toBeInTheDocument();
      } else {
        rerender(<SourceBrowser accountId="b" />);
        await screen.findByText("Song b");
      }
      expect(held.signal?.aborted).toBe(true);
      // A successful server result does not restore this UI attempt ownership.
      await act(async () => { held.release({ data: { playSelectedMedia: "late-occurrence" } }); });
      expect(screen.queryByText("Playback requested.")).toBeNull();
      expect(invalidated).not.toHaveBeenCalled();
      expect(sent).toHaveBeenCalledTimes(1);
      expect(forbidden).not.toHaveBeenCalled();
    } finally { held.release({ data: { playSelectedMedia: "late-occurrence" } }); }
  });

  it.each(["unsupported", "transport", "bad acknowledgement"] as const)("shows a %s playback error without retry or ID-only fallback", async (failure) => {
    accounts();
    const sent = vi.fn();
    const forbidden = vi.fn();
    server.use(
      graphql.query("BrowseSourceMedia", () => HttpResponse.json(body("a", [entry("a", "Song")]))),
      graphql.mutation("PlaySelectedSourceMedia", ({ variables }) => {
        sent(variables);
        return failure === "unsupported" ? HttpResponse.json({ errors: [{ message: "Selected media playback is unavailable on this remote receiver" }] })
          : failure === "transport" ? new HttpResponse(null, { status: 503 })
          : HttpResponse.json({ data: { playSelectedMedia: "" } });
      }),
      graphql.mutation(/^(AddSourceTracks|PlayTrack|PlayTrackAt|PlayNext)$/, () => {
        forbidden(); return HttpResponse.json({ errors: [{ message: "Unexpected fallback" }] });
      }),
    );
    const { user } = renderWithProviders(<SourceBrowser accountId="a" />);
    await user.click(await screen.findByRole("button", { name: "Play next: Song" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(failure === "unsupported"
      ? "Selected media playback is unavailable on this remote receiver" : "Request outcome unknown");
    expect(screen.queryByText("Track added to play next.")).toBeNull();
    expect(screen.getByRole("button", { name: "Play next: Song" })).toBeEnabled();
    expect(sent).toHaveBeenCalledTimes(1);
    expect(sent.mock.calls[0][0].next).toBe(true);
    expect(forbidden).not.toHaveBeenCalled();
  });

  it("cancels an incomplete batch and rejects partial GraphQL pages without submitting any tracks", async () => {
    accounts();
    const queued = vi.fn();
    const first = entry("a", "First page");
    server.use(
      graphql.query("BrowseSourceMedia", ({ variables }) => HttpResponse.json(variables.offset === "0"
        ? body("a", [first], "100")
        : { ...body("a", [entry("a", "Partial")]), errors: [{ message: "Second page failed" }] })),
      graphql.mutation("AddSourceTracks", ({ variables }) => { queued(variables); return HttpResponse.json({ data: { addTracks: true } }); }),
    );
    const held = heldBody();
    const original = globalThis.fetch;
    let delayNext = true;
    vi.spyOn(globalThis, "fetch").mockImplementation((input, init) => {
      const request = JSON.parse(String(init?.body ?? "{}"));
      if (delayNext && request.query?.includes("query BrowseSourceMedia(") && request.variables.offset === "100") {
        delayNext = false; return held.response(init);
      }
      return original(input, init);
    });
    const { user } = renderWithProviders(<SourceBrowser accountId="a" />);
    try {
      await screen.findByText("First page");
      await user.click(screen.getByRole("button", { name: "Add all audio in this view" }));
      await held.started;
      await user.click(screen.getByRole("button", { name: "Cancel waiting" }));
      expect(held.signal?.aborted).toBe(true);
      await act(async () => { held.release(body("a", [entry("a", "Late page")])); });
      expect(queued).not.toHaveBeenCalled();
      await user.click(screen.getByRole("button", { name: "Refresh view" }));
      await screen.findByText("First page");
      await user.click(screen.getByRole("button", { name: "Add all audio in this view" }));
      expect(await screen.findByRole("alert")).toHaveTextContent("Second page failed");
      expect(queued).not.toHaveBeenCalled();
    } finally { held.release(body("a", [])); }
  });

  it.each(["disable", "rescan"] as const)("revokes a pending batch through the generated %s hook and rechecks the account", async (operation) => {
    accounts();
    let enabled = true;
    let title = "Before extension change";
    let finishMutation!: () => void;
    const mutationWait = new Promise<void>((resolve) => { finishMutation = resolve; });
    const mutationEntered = vi.fn();
    const queued = vi.fn();
    const extension = {
      id: "fixture", name: "Fixture", version: "1", author: null, description: null,
      homepage: null, repository: null, license: null, logo: null, topics: [],
      capabilities: ["source"], allowedHosts: [], libraryRead: false, status: "disabled", path: "fixture.wasm",
    };
    server.use(
      graphql.query("ExtensionSourceDefinitions", () => HttpResponse.json({ data: { extensionSourceDefinitions: [{
        extensionId: "fixture", name: "Fixture", enabled, sourceApiVersion: 1, configKeys: [],
      }] } })),
      graphql.query("BrowseSourceMedia", () => HttpResponse.json(body("a", [entry("a", title)], "100"))),
      graphql.mutation("AddSourceTracks", ({ variables }) => {
        queued(variables); return HttpResponse.json({ data: { addTracks: true } });
      }),
      graphql.mutation(operation === "disable" ? "SetExtensionEnabled" : "RescanExtensions", async ({ variables }) => {
        mutationEntered(variables);
        await mutationWait;
        if (operation === "disable") enabled = false;
        title = "After extension change";
        return HttpResponse.json({ data: operation === "disable"
          ? { setExtensionEnabled: extension } : { rescanExtensions: [] } });
      }),
    );
    const held = heldBody();
    const original = globalThis.fetch;
    vi.spyOn(globalThis, "fetch").mockImplementation((input, init) => {
      const request = JSON.parse(String(init?.body ?? "{}"));
      if (request.query?.includes("query BrowseSourceMedia(") && request.variables.offset === "100") {
        return held.response(init);
      }
      return original(input, init);
    });
    const { user } = renderWithProviders(<><SourceBrowser accountId="a" /><ExtensionControls /></>);
    try {
      await screen.findByText("Before extension change");
      await user.click(screen.getByRole("button", { name: "Add all audio in this view" }));
      await waitFor(() => expect(held.signal).toBeDefined());
      await user.click(screen.getByRole("button", { name: operation === "disable" ? "Disable extension" : "Rescan extensions" }));
      await waitFor(() => expect(mutationEntered).toHaveBeenCalledTimes(1));
      expect(held.signal?.aborted).toBe(true);
      expect(screen.queryByText("Before extension change")).toBeNull();
      expect(screen.getByText("Loading source account…")).toBeInTheDocument();
      // Complete the old batch while the extension mutation is still pending.
      // It must not submit even though this body reader ignores AbortSignal.
      await act(async () => { held.release(body("a", [entry("a", "Obsolete batch")])); });
      expect(queued).not.toHaveBeenCalled();
      await act(async () => { finishMutation(); });
      if (operation === "disable") {
        await screen.findByText(/This source is disabled or no longer installed/);
        expect(screen.queryByRole("button", { name: "Add all audio in this view" })).toBeNull();
        expect(queued).not.toHaveBeenCalled();
      } else {
        await screen.findByText("After extension change");
        expect(screen.queryByText("Obsolete batch")).toBeNull();
        expect(queued).not.toHaveBeenCalled();
        await user.click(screen.getByRole("button", { name: "Add to queue: After extension change" }));
        await screen.findByText("Track added to queue.");
        expect(queued).toHaveBeenCalledTimes(1);
        expect(queued.mock.calls[0][0].tracks[0].title).toBe("After extension change");
      }
    } finally {
      held.release(body("a", []));
      finishMutation();
    }
  });

  it("revokes an account A batch across A to B to A, even if its JSON completes after abort", async () => {
    accounts();
    const queued = vi.fn();
    server.use(
      graphql.query("BrowseSourceMedia", ({ variables }) => HttpResponse.json(body(String(variables.accountId), [entry(String(variables.accountId), "Current " + variables.accountId)]))),
      graphql.mutation("AddSourceTracks", ({ variables }) => { queued(variables); return HttpResponse.json({ data: { addTracks: true } }); }),
    );
    const held = heldBody();
    const original = globalThis.fetch;
    let holdBatch = false;
    vi.spyOn(globalThis, "fetch").mockImplementation((input, init) => {
      if (holdBatch && String(init?.body).includes("query BrowseSourceMedia(")) {
        holdBatch = false; return held.response(init);
      }
      return original(input, init);
    });
    const { user, rerender } = renderWithProviders(<SourceBrowser accountId="a" />);
    try {
      await screen.findByText("Current a");
      holdBatch = true;
      await user.click(screen.getByRole("button", { name: "Add all audio in this view" }));
      await held.started;
      rerender(<SourceBrowser accountId="b" />);
      await screen.findByText("Current b");
      expect(screen.queryByText("Current a")).toBeNull();
      rerender(<SourceBrowser accountId="a" />);
      await screen.findByText("Current a");
      expect(held.signal?.aborted).toBe(true);
      await act(async () => { held.release(body("a", [entry("a", "Obsolete A result")])); });
      expect(screen.queryByText("Obsolete A result")).toBeNull();
      expect(screen.queryByText("Current b")).toBeNull();
      expect(queued).not.toHaveBeenCalled();
      await user.click(screen.getByRole("button", { name: "Add to queue: Current a" }));
      await screen.findByText(/Track added/);
      expect(queued).toHaveBeenCalledTimes(1);
    } finally { held.release(body("a", [])); }
  });
});
