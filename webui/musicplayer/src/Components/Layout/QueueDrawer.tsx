import { useAtom } from "jotai";
import { useState } from "react";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { queueOpenAtom } from "../../State";
import type { Track } from "../../Types";
import { Artwork, IconButton, Icons, cn } from "../UI";

type QueueTab = "queue" | "history";

/**
 * The desktop's queue drawer: a now-playing card, what is up next, and the
 * history behind a second tab.
 *
 * On a phone it becomes a full-height sheet rather than a 320px rail — a
 * fixed-width drawer over a 390px viewport leaves nothing behind it.
 */
const QueueDrawer = () => {
  const [open, setOpen] = useAtom(queueOpenAtom);
  const [tab, setTab] = useState<QueueTab>("queue");
  const { formatTime } = useTimeFormat();
  const { nowPlaying, nextTracks, previousTracks, playTrackAt, removeTrackAt } =
    usePlayback();

  if (!open) return null;

  const rows = tab === "queue" ? nextTracks : previousTracks;
  const total = nextTracks.length + previousTracks.length;

  const Row = ({ track, position }: { track: Track; position: number }) => (
    <div className="group flex h-[46px] items-center gap-[10px] px-[14px] hover:bg-hover">
      <button
        type="button"
        onClick={() => playTrackAt({ position })}
        className="flex min-w-0 flex-1 items-center gap-[10px] text-left"
      >
        <span className="w-6 shrink-0 text-right font-mono text-[11px] text-muted">
          {position + 1}
        </span>
        <span className="flex min-w-0 flex-1 flex-col gap-[2px]">
          <span className="truncate text-xs text-fg">{track.title}</span>
          <span className="truncate text-[10px] text-dim">{track.artist}</span>
        </span>
        <span className="shrink-0 font-mono text-[10px] text-muted">
          {/* `Track.duration` is in seconds; `formatTime` takes milliseconds. */}
          {formatTime(track.duration * 1000)}
        </span>
      </button>
      <IconButton
        icon={Icons.close}
        iconSize={13}
        size={26}
        aria-label={`Remove ${track.title} from the queue`}
        data-testid={`remove-${position}`}
        className="opacity-0 transition-opacity group-hover:opacity-100"
        onClick={() => removeTrackAt({ position })}
      />
    </div>
  );

  return (
    <>
      {/* Below lg the sheet covers the page, so it needs a scrim to close on.
          Hidden from assistive tech: it is a pointer affordance, and the
          drawer's own close button is the keyboard route out. */}
      <button
        type="button"
        aria-hidden="true"
        tabIndex={-1}
        onClick={() => setOpen(false)}
        className="fixed inset-0 z-30 bg-black/50 lg:hidden"
      />
      <aside
        className={cn(
          "fixed inset-y-0 right-0 z-30 flex w-full flex-col border-l border-line bg-panel",
          "sm:w-[360px] lg:static lg:z-auto lg:w-[320px] lg:shrink-0"
        )}
      >
        <div className="flex items-center gap-2 p-4 pb-[10px]">
          <h2 className="flex-1 text-[15px] font-bold text-fg">Queue</h2>
          <span className="font-mono text-[11px] text-muted">
            {total === 1 ? "1 track" : `${total} tracks`}
          </span>
          <IconButton
            icon={Icons.close}
            iconSize={15}
            aria-label="Close queue"
            onClick={() => setOpen(false)}
          />
        </div>

        <div className="flex gap-2 px-4 pb-[10px]">
          {(["queue", "history"] as const).map((value) => (
            <button
              key={value}
              type="button"
              onClick={() => setTab(value)}
              className={cn(
                "h-7 flex-1 rounded-full text-[11px] transition-colors",
                tab === value
                  ? "border border-accent/40 bg-selected font-semibold text-accent"
                  : "text-dim hover:bg-hover"
              )}
            >
              {value === "queue" ? "Play Queue" : "History"}
            </button>
          ))}
        </div>
        <div className="h-px bg-line" />

        {tab === "queue" && nowPlaying?.title && (
          <div className="flex flex-col gap-2 p-3 pb-2">
            <span className="text-[9px] tracking-[1.5px] text-muted">
              NOW PLAYING
            </span>
            <div className="flex h-16 items-center gap-[10px] rounded-control border border-accent/30 bg-selected p-[10px]">
              <Artwork
                src={nowPlaying.cover}
                alt={nowPlaying.title}
                fallbackIcon={Icons.music}
                iconSize={18}
                className="size-11"
              />
              <span className="flex min-w-0 flex-1 flex-col gap-[2px]">
                <span className="truncate text-xs font-semibold text-accent">
                  {nowPlaying.title}
                </span>
                <span className="truncate text-[10px] text-dim">
                  {nowPlaying.artist}
                </span>
              </span>
              {nowPlaying.isPlaying ? (
                <Icons.play size={13} className="text-accent" />
              ) : (
                <Icons.pause size={13} className="text-accent" />
              )}
            </div>
          </div>
        )}

        {tab === "queue" && (
          <div className="flex items-center px-3 pb-1 pt-[6px]">
            <span className="flex-1 text-[9px] tracking-[1.5px] text-muted">
              UP NEXT
            </span>
            <span className="font-mono text-[9px] text-muted">
              {nextTracks.length === 1
                ? "1 track"
                : `${nextTracks.length} tracks`}
            </span>
          </div>
        )}

        <div className="scrollbar-skin min-h-0 flex-1 overflow-y-auto">
          {rows.length === 0 ? (
            <p className="grid h-full place-items-center px-4 text-xs text-muted">
              {tab === "queue" ? "Nothing up next" : "No history yet"}
            </p>
          ) : (
            rows.map((track, position) => (
              <Row
                key={`${track.id}-${position}`}
                track={track}
                position={position}
              />
            ))
          )}
        </div>
      </aside>
    </>
  );
};

export default QueueDrawer;
