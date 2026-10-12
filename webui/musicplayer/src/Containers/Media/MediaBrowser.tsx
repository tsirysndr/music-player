import { useIsMutating, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
import { Button, EmptyState, Icons } from "../../Components/UI";
import { useGetConnectedServerQuery } from "../../Hooks/GraphQL";
import {
  browserKey, fetchMediaBrowser, isSourceMutation, useMediaPages, type MediaEntry,
} from "./api";
import { errorMessage, useMediaQueue } from "./useMediaQueue";
import AudioChoiceEditor from "./AudioChoiceEditor";
import type { AudioChoice } from "./api";

export default function MediaBrowser() {
  const client = useQueryClient();
  const connected = useGetConnectedServerQuery();
  const switching = useIsMutating({
    predicate: (mutation) => isSourceMutation(mutation.options.mutationKey),
  }) > 0;
  const account = connected.data?.connectedServer?.id ?? null;
  const browser = useQuery({
    queryKey: browserKey(account),
    queryFn: ({ signal }) => fetchMediaBrowser(signal),
    enabled: connected.isSuccess && !switching,
    placeholderData: undefined,
    retry: false,
  });
  const refreshingSource = connected.isFetching &&
    client.getQueryState(["GetConnectedServer"])?.isInvalidated;
  const refreshingBrowser = browser.isFetching &&
    client.getQueryState(browserKey(account))?.isInvalidated;

  if (switching || connected.isPending || refreshingSource ||
      (connected.isSuccess && browser.isPending) || refreshingBrowser) {
    return <p role="status" className="py-8 text-sm text-muted">Loading media libraries…</p>;
  }
  if (connected.isError || browser.isError) {
    return (
      <div className="space-y-3 py-8">
        <p role="alert">{errorMessage(connected.error ?? browser.error)}</p>
        <Button variant="outline" onClick={() => {
          void connected.refetch();
          void browser.refetch();
        }}>Retry</Button>
      </div>
    );
  }
  const available = browser.data?.mediaBrowser;
  if (!available) {
    return <EmptyState icon={Icons.folder} title="Connect a server to browse media libraries"
      action={<Link className="text-accent underline" to="/servers">Servers</Link>} />;
  }
  if (available.serverId !== account) {
    return <EmptyState title="The browsing server changed" action={
      <Button variant="outline" onClick={() => { void connected.refetch(); }}>Refresh libraries</Button>
    } />;
  }
  if (!available.supported) {
    return <EmptyState icon={Icons.folder} title="This server does not support media library navigation" />;
  }
  // Account changes unmount the entire path/operation state, not only the rows.
  return <AccountMedia key={available.serverId} serverId={available.serverId} />;
}

function AccountMedia({ serverId }: { serverId: string }) {
  const [path, setPath] = useState<{ id: string; title: string }[]>([]);
  const parent = path.at(-1)?.id ?? null;
  const pages = useMediaPages(serverId, parent);
  const actions = useMediaQueue(serverId);
  const entries = pages.data?.pages.flatMap((page) => page.entries) ?? [];
  const navigate = (next: typeof path) => {
    if (actions.phase !== "idle") actions.cancel();
    setPath(next);
  };

  return (
    <section aria-label="Media browser" className="space-y-4 py-4">
      <nav aria-label="Media path">
        <ol className="flex flex-wrap items-center gap-2 text-sm">
          <li>{path.length ? (
            <Button variant="ghost" onClick={() => navigate([])}>Libraries</Button>
          ) : <span aria-current="page">Libraries</span>}</li>
          {path.map((part, index) => (
            <li key={part.id} className="flex items-center gap-2">
              <span aria-hidden="true" className="text-muted">/</span>
              {index === path.length - 1 ? (
                <span aria-current="page" className="break-words">{part.title}</span>
              ) : (
                <Button variant="ghost" onClick={() => navigate(path.slice(0, index + 1))}>
                  {part.title}
                </Button>
              )}
            </li>
          ))}
        </ol>
      </nav>
      <p className="text-xs text-muted">Listen to audio from music, movies and episodes.</p>
      {actions.phase === "reading" && (
        <div className="flex flex-wrap items-center gap-3">
          <p role="status" className="text-sm text-muted">Reading all container tracks…</p>
          <Button variant="outline" onClick={actions.cancel}>Cancel waiting</Button>
        </div>
      )}
      {actions.phase === "submitting" && (
        <p role="status" className="text-sm text-muted">Updating queue… The request has been sent.</p>
      )}
      {actions.message && <p role="status" className="text-sm text-muted">{actions.message}</p>}
      {actions.error && <p role="alert" className="text-sm">{actions.error}</p>}
      {pages.isPending && <p role="status" className="text-sm text-muted">Loading items…</p>}
      {pages.isError && (
        <div className="space-y-2">
          <p role="alert">{errorMessage(pages.error)}</p>
          <Button variant="outline" onClick={() => {
            if (pages.isFetchNextPageError) void pages.fetchNextPage();
            else void pages.refetch();
          }}>Retry loading items</Button>
        </div>
      )}
      {pages.isSuccess && !entries.length && <EmptyState title="No items in this container" />}
      <ul className="divide-y divide-line">
        {entries.map((entry) => (
          <MediaRow key={entry.id} entry={entry} busy={actions.phase === "submitting"}
            onOpen={() => navigate([...path, { id: entry.id, title: entry.title }])}
            onQueue={() => { void actions.queueContainer(entry.id); }}
            onPlay={() => { if (entry.track) void actions.leaf(entry.track.id, false); }}
            onAudio={(choice) => { if (entry.track) void actions.selected(entry.track, choice); }}
            onNext={() => { if (entry.track) void actions.leaf(entry.track.id, true); }} />
        ))}
      </ul>
      {pages.hasNextPage && (
        <Button variant="outline" disabled={pages.isFetching} onClick={() => { void pages.fetchNextPage(); }}>
          {pages.isFetchingNextPage ? "Loading more…" : "Load more"}
        </Button>
      )}
    </section>
  );
}

function MediaRow({ entry, busy, onOpen, onQueue, onPlay, onNext, onAudio }: {
  entry: MediaEntry; busy: boolean;
  onOpen: () => void; onQueue: () => void; onPlay: () => void; onNext: () => void;
  onAudio: (choice: AudioChoice) => void;
}) {
  const [choosing, setChoosing] = useState(false);
  const playable = !entry.isContainer && entry.track &&
    (entry.mediaType === "Audio" || entry.mediaType === "Video");
  // Keep decimal strings intact: season indices can exceed JS's exact integers.
  const detail = [entry.itemType, entry.mediaType,
    entry.seasonNumber !== null ? `Season ${entry.seasonNumber}` : null,
    entry.episodeNumber !== null ? `Episode ${entry.episodeNumber}` : null,
  ].filter(Boolean).join(" · ");
  return (
    <li className="flex flex-wrap items-center gap-3 py-3">
      <div className="min-w-0 flex-1 basis-52">
        {entry.isContainer ? (
          <button type="button" onClick={onOpen} aria-label={`Open ${entry.title}`}
            className="flex items-center gap-2 text-left text-sm hover:text-accent">
            <Icons.folder size={18} className="shrink-0 text-muted" aria-hidden="true" />
            <span className="break-words">{entry.title}</span>
          </button>
        ) : <p className="break-words text-sm">{entry.title}</p>}
        <p className="mt-1 break-words text-xs text-muted">{detail}</p>
      </div>
      {entry.isContainer ? (
        <Button variant="outline" disabled={busy} onClick={onQueue}
          aria-label={`Add all to queue: ${entry.title}`}>Add all to queue</Button>
      ) : playable ? (
        <div className="flex flex-wrap gap-2">
          <Button icon={Icons.play} disabled={busy} onClick={onPlay}
            aria-label={`Listen to ${entry.title}`}>Listen</Button>
          <Button variant="ghost" disabled={busy} onClick={onNext}
            aria-label={`Play next: ${entry.title}`}>Play next</Button>
          <Button variant="outline" disabled={busy} onClick={() => setChoosing(!choosing)}
            aria-expanded={choosing} aria-label={`Choose audio: ${entry.title}`}>Audio tracks</Button>
        </div>
      ) : null}
      {choosing && playable && entry.track && <div className="w-full">
        <AudioChoiceEditor key={entry.track.id} source={entry.track.id} busy={busy} onSubmit={onAudio} />
      </div>}
    </li>
  );
}
