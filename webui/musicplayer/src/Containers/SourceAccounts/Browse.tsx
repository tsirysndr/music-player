import { useInfiniteQuery, useQuery, useQueryClient } from "@tanstack/react-query";
import { useSetAtom } from "jotai";
import { useLayoutEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { AppShell } from "../../Components/Layout";
import { Button } from "../../Components/UI";
import {
  GET_EXTENSION_SOURCE_DEFINITIONS, GET_SOURCE_ACCOUNT,
  type ExtensionSourceAccount, type ExtensionSourceDefinitionsQuery,
  type SourceAccountQuery, type SourceBrowseList, type SourceMediaEntry, type SourceMediaTrack,
} from "../../GraphQL/SourceAccounts/Query";
import { queueOpenAtom } from "../../State";
import AudioChoiceEditor from "../Media/AudioChoiceEditor";
import type { AudioChoice } from "../Media/api";
import { enqueueSelectedSource, enqueueSourceTracks, playSourceTrack, readAllViewTracks, readSourcePage } from "./media";
import { errorMessage, sourceRequest, useSourceOperation } from "./requests";

const lists: { value: SourceBrowseList; label: string }[] = [
  { value: "TRACKS", label: "Tracks" }, { value: "ALBUMS", label: "Albums" },
  { value: "ARTISTS", label: "Artists" }, { value: "PLAYLISTS", label: "Playlists" },
];

export default function SourceBrowsePage() {
  const { accountId } = useParams<{ accountId: string }>();
  return <AppShell title="Extension source media">
    {accountId ? <SourceBrowser accountId={accountId} /> : <p role="alert">No source account selected.</p>}
  </AppShell>;
}

export function SourceBrowser({ accountId }: { accountId: string }) {
  // Even A → B → A mounts fresh intent state; matching IDs cannot revive A's
  // previous read or queued continuation.
  return <AccountGate key={accountId} accountId={accountId} />;
}

function AccountGate({ accountId }: { accountId: string }) {
  const client = useQueryClient();
  const account = useQuery({
    queryKey: ["ExtensionSourceAccount", accountId],
    queryFn: async ({ signal }) => {
      const [row, definitions] = await Promise.all([
        sourceRequest<SourceAccountQuery>(GET_SOURCE_ACCOUNT, { id: accountId }, signal),
        sourceRequest<ExtensionSourceDefinitionsQuery>(GET_EXTENSION_SOURCE_DEFINITIONS, {}, signal),
      ]);
      if (row.sourceAccount.id !== accountId) throw new Error("The response belongs to another source account");
      return { row: row.sourceAccount, definition: definitions.extensionSourceDefinitions.find(
        (item) => item.extensionId === row.sourceAccount.extensionId,
      ) };
    },
    retry: false, gcTime: 0, placeholderData: undefined,
  });
  const [generation, setGeneration] = useState(0);
  const [switching, setSwitching] = useState(false);
  useLayoutEffect(() => {
    let alive = true;
    let revision = 0;
    const relevant = (key: unknown) => key === "SetExtensionEnabled" || key === "RescanExtensions";
    const cache = client.getMutationCache();
    const pending = new Set(cache.getAll().filter((mutation) =>
      relevant(mutation.options.mutationKey?.[0]) && mutation.state.status === "pending",
    ).map((mutation) => mutation.mutationId));
    if (pending.size) setSwitching(true);
    const unsubscribe = cache.subscribe((event) => {
      if (event.type !== "updated" || !relevant(event.mutation.options.mutationKey?.[0])) return;
      if (event.action.type === "pending") {
        pending.add(event.mutation.mutationId);
        revision += 1;
        setSwitching(true);
        setGeneration((old) => old + 1);
      } else if (event.action.type === "success" || event.action.type === "error") {
        pending.delete(event.mutation.mutationId);
        const expected = ++revision;
        if (pending.size === 0) {
          const finish = () => { if (alive && expected === revision) setSwitching(false); };
          void client.invalidateQueries({ queryKey: ["ExtensionSourceAccount", accountId] }).then(finish, finish);
        }
      }
    });
    return () => { alive = false; unsubscribe(); };
  }, [client, accountId]);
  const refreshing = account.isFetching && client.getQueryState(["ExtensionSourceAccount", accountId])?.isInvalidated;
  return <section className="space-y-4 py-4">
    <Link to="/source-accounts" className="text-sm text-accent underline">Manage or switch source accounts</Link>
    {switching || account.isPending || refreshing ? <p role="status">Loading source account…</p>
      : account.isError ? <div className="space-y-2">
        <p role="alert">{errorMessage(account.error)}</p>
        <Button variant="outline" onClick={() => { void account.refetch(); }}>Retry account</Button>
      </div>
      : !account.data.definition?.enabled ? <p role="status">This source is disabled or no longer installed. Its account can still be managed.</p>
      : !account.data.row.identityBound ? <p role="status">Connect this saved account before browsing.</p>
      : ![1, 2].includes(account.data.row.sourceApiVersion) ? <p role="alert">This source protocol is not supported.</p>
      : <AccountBrowse key={account.data.row.configRevision + ":" + generation} account={account.data.row} />}
  </section>;
}

function AccountBrowse({ account }: { account: ExtensionSourceAccount }) {
  const client = useQueryClient();
  const openQueue = useSetAtom(queueOpenAtom);
  const actions = useSourceOperation();
  const [path, setPath] = useState<{ id: string; title: string }[]>([]);
  const [list, setList] = useState<SourceBrowseList>("TRACKS");
  const [search, setSearch] = useState("");
  const [query, setQuery] = useState("");
  const [epoch, setEpoch] = useState(0);
  const [paused, setPaused] = useState(false);
  const [choiceTrack, setChoiceTrack] = useState<SourceMediaTrack | null>(null);
  const [choiceAction, setChoiceAction] = useState<"queue" | "listen" | "next">("queue");
  const legacy = account.sourceApiVersion === 1;
  const parent = path.at(-1)?.id ?? null;
  const variables = { accountId: account.id, parent, list: legacy ? list : null, query: query || null, limit: 100 };
  const pageKey = ["BrowseSourceMedia", account.id, account.configRevision, parent, legacy ? list : null, query, epoch];
  const pages = useInfiniteQuery({
    queryKey: pageKey,
    initialPageParam: "0",
    queryFn: ({ pageParam, signal }) => readSourcePage({ ...variables, offset: pageParam }, signal),
    getNextPageParam: (page) => page.nextOffset ?? undefined,
    enabled: !paused, retry: false, gcTime: 0, placeholderData: undefined, refetchOnWindowFocus: false,
  });
  const entries = paused ? [] : pages.data?.pages.flatMap((page) => page.entries) ?? [];
  const cancelRead = () => { void client.cancelQueries({ queryKey: ["BrowseSourceMedia", account.id] }); };
  const invalidateIntent = () => {
    // Synchronous revoke before a state/navigation update or another request.
    actions.cancel(); cancelRead(); setChoiceTrack(null);
  };
  const navigate = (next: typeof path, nextList = list) => {
    invalidateIntent(); setPath(next); setList(nextList); setSearch(""); setQuery(""); setPaused(false); setEpoch((old) => old + 1);
  };
  // Extension disable/rescan can start in another mounted surface. Revoke
  // ownership synchronously at that intent, before React replaces this view.
  useLayoutEffect(() => client.getMutationCache().subscribe((event) => {
    if (event.type !== "updated" || event.action.type !== "pending") return;
    const key = event.mutation.options.mutationKey?.[0];
    if (key === "SetExtensionEnabled" || key === "RescanExtensions") actions.cancel();
  }), [client, actions.cancel]);
  const afterQueue = () => {
    void client.invalidateQueries({ queryKey: ["GetTracklist"] });
    void client.invalidateQueries({ queryKey: ["MediaQueue"] });
  };
  const queueTrack = (track: SourceMediaTrack, choice?: AudioChoice) => {
    void actions.run((attempt) => attempt.submit(() => choice === undefined
      ? enqueueSourceTracks([track], attempt.signal)
      : enqueueSelectedSource(track, choice, attempt.signal)), () => {
      afterQueue();
    }, "Track added to queue.");
  };
  const playTrack = (track: SourceMediaTrack, next: boolean, choice?: AudioChoice) => {
    void actions.run((attempt) => attempt.submit(() =>
      playSourceTrack(track, next, attempt.signal, choice)), afterQueue,
    next ? "Track added to play next." : "Playback requested.");
  };
  const queueView = () => {
    void actions.run(async (attempt) => {
      const tracks = await readAllViewTracks(variables, attempt.signal);
      if (!attempt.current()) return 0;
      if (tracks.length) await attempt.submit(() => enqueueSourceTracks(tracks, attempt.signal));
      return tracks.length;
    }, (count) => { if (count) afterQueue(); }, "Finished adding audio from this view.");
  };

  return <div aria-label="Source media browser" className="space-y-4">
    <h1 className="break-words text-xl font-semibold">{account.name}</h1>
    <nav aria-label="Source media path" className="flex flex-wrap items-center gap-2">
      <Button variant="ghost" onClick={() => navigate([])}>Root</Button>
      {path.map((crumb, index) => <Button key={index + ":" + crumb.id} variant="ghost"
        className="h-auto min-h-[34px] whitespace-normal break-words text-left"
        onClick={() => navigate(path.slice(0, index + 1))}>{crumb.title}</Button>)}
    </nav>
    {legacy && <div aria-label="Source list" className="flex flex-wrap gap-2">
      {lists.map((item) => <Button key={item.value} variant={list === item.value ? "accent" : "outline"}
        aria-pressed={list === item.value} onClick={() => navigate(path, item.value)}>{item.label}</Button>)}
    </div>}
    <form role="search" className="flex flex-wrap gap-2" onSubmit={(event) => {
      event.preventDefault(); invalidateIntent(); setQuery(search); setPaused(false); setEpoch((old) => old + 1);
    }}>
      <label className="min-w-0 flex-1 text-sm">Search this source view
        <input value={search} onChange={(event) => setSearch(event.target.value)}
          className="w-full rounded-control border border-line bg-panel p-2 text-fg" />
      </label>
      <Button type="submit" variant="outline">Search</Button>
      <Button variant="outline" onClick={() => {
        invalidateIntent(); setPaused(false); setEpoch((old) => old + 1);
      }}>Refresh view</Button>
    </form>
    <p className="text-xs text-muted">Listen to audio from this source.</p>
    {legacy && <p className="text-xs text-muted">This source provides four lists. Choose the list to open inside a container. Audio version selection is not available.</p>}
    <div className="flex flex-wrap gap-2">
      <Button disabled={paused || actions.busy || (legacy && list !== "TRACKS")}
        onClick={queueView}>Add all audio in this view</Button>
      <Button variant="outline" onClick={() => openQueue(true)}>Open play queue</Button>
      {(pages.isFetching || actions.busy) && <Button variant="outline" onClick={() => {
        invalidateIntent(); setPaused(true);
      }}>Cancel waiting</Button>}
    </div>
    <p className="text-xs text-muted">Open a collection to browse its contents.</p>
    {actions.busy && <p role="status">Please wait…</p>}
    {actions.error && <p role="alert">{actions.error}</p>}
    {actions.message && <p role="status">{actions.message}</p>}
    {paused && <p role="status">Cancelled. Refresh to load this view again.</p>}
    {!paused && pages.isPending && <p role="status">Loading source media…</p>}
    {!paused && pages.isError && <div className="space-y-2">
      <p role="alert">{errorMessage(pages.error)}</p>
      <Button variant="outline" onClick={() => {
        if (pages.isFetchNextPageError) void pages.fetchNextPage(); else void pages.refetch();
      }}>Retry media</Button>
    </div>}
    {!paused && pages.isSuccess && entries.length === 0 && <p>No media in this view.</p>}
    <ul className="space-y-3">
      {entries.map((entry, index) => <li key={entry.id + ":" + index} className="space-y-2 rounded-skin bg-panel p-4">
        <h2 className="whitespace-normal break-words font-semibold">{entry.title}</h2>
        <EntryLabels entry={entry} />
        {entry.isContainer ? <div className="flex flex-wrap gap-2">
          {legacy ? lists.map((item) => <Button key={item.value} variant="outline"
            aria-label={"Open " + item.label.toLowerCase() + " in " + entry.title}
            onClick={() => navigate([...path, { id: entry.id, title: entry.title }], item.value)}>Open {item.label.toLowerCase()}</Button>)
            : <Button variant="outline" aria-label={"Open " + entry.title}
              onClick={() => navigate([...path, { id: entry.id, title: entry.title }])}>Open</Button>}
        </div> : entry.track ? <div className="flex flex-wrap gap-2">
          <Button disabled={actions.busy} aria-label={"Add to queue: " + entry.title}
            onClick={() => queueTrack(entry.track!)}>Add to queue</Button>
          <Button variant="outline" disabled={actions.busy} aria-label={"Listen: " + entry.title}
            onClick={() => playTrack(entry.track!, false)}>Listen</Button>
          <Button variant="outline" disabled={actions.busy} aria-label={"Play next: " + entry.title}
            onClick={() => playTrack(entry.track!, true)}>Play next</Button>
          {!legacy && <Button variant="ghost" disabled={actions.busy} aria-label={"Audio options: " + entry.title}
            onClick={() => { actions.cancel(); setChoiceAction("queue"); setChoiceTrack(entry.track); }}>Audio options</Button>}
        </div> : <p className="text-xs text-muted">No playable audio supplied by this source.</p>}
        {!entry.isContainer && !legacy && choiceTrack && entry.track && choiceTrack.id === entry.track.id && <>
          <Button variant="ghost" onClick={() => { actions.cancel(); setChoiceTrack(null); }}>Close audio options</Button>
          <label className="block text-sm">With selected audio
            <select aria-label="With selected audio" value={choiceAction} disabled={actions.busy}
              className="w-full rounded-control border border-line bg-panel p-2 text-sm"
              onChange={(event) => setChoiceAction(event.target.value as typeof choiceAction)}>
              <option value="queue">Add to queue</option>
              <option value="listen">Listen</option>
              <option value="next">Play next</option>
            </select>
          </label>
          <AudioChoiceEditor key={choiceTrack.id} source={choiceTrack.id} busy={actions.busy}
            submitLabel={choiceAction === "queue" ? "Add with this audio" : choiceAction === "listen" ? "Listen with this audio" : "Play next with this audio"}
            onSubmit={(choice) => choiceAction === "queue" ? queueTrack(choiceTrack, choice)
              : playTrack(choiceTrack, choiceAction === "next", choice)} />
        </>}
      </li>)}
    </ul>
    {!paused && pages.hasNextPage && <Button variant="outline" disabled={pages.isFetching}
      onClick={() => { void pages.fetchNextPage(); }}>Load more</Button>}
  </div>;
}

function EntryLabels({ entry }: { entry: SourceMediaEntry }) {
  const labels = [entry.itemType, entry.mediaType,
    entry.seasonNumber !== null ? "Season " + entry.seasonNumber : null,
    entry.episodeNumber !== null ? "Episode " + entry.episodeNumber : null];
  const track = entry.track;
  return <div className="space-y-1 break-words text-xs text-muted">
    <p>{labels.filter(Boolean).join(" · ")}</p>
    {track && <p>{[track.artist, track.album.title, track.album.artist,
      track.album.year !== null ? String(track.album.year) : null].filter(Boolean).join(" · ")}</p>}
  </div>;
}
