import { Link, useNavigate } from "react-router-dom";
import cn from "./cn";
import ContextMenu, {
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
} from "./ContextMenu";
import IconButton from "./IconButton";
import { Icons } from "./icons";
import { keyColorFor } from "./keyColor";
import { useLibraryAnalysis } from "./LibraryAnalysis";
import LikeButton from "./LikeButton";

export type TrackRowItem = {
  id: string;
  title: string;
  artist: string;
  artistId?: string | null;
  album?: string | null;
  albumId?: string | null;
  /** Formatted mm:ss — the row never does the arithmetic itself. */
  duration: string;
  /** Metadata track number; album views show this instead of the position. */
  trackNumber?: number | null;
  /** Which disc it sits on. Only meaningful on a multi-disc album. */
  discNumber?: number | null;
  liked?: boolean;
  /** Camelot notation, e.g. "8A". Absent when the track has not been analysed. */
  key?: string | null;
  bpm?: number | null;
};

export type PlaylistOption = { id: string; name: string };

export type TrackRowProps = {
  track: TrackRowItem;
  /** 0-based position in the list, shown when there is no track number. */
  index: number;
  /** Show the metadata track number rather than the row position. */
  useTrackNumber?: boolean;
  current?: boolean;
  /** The first few playlists, offered inline in the "…" menu. */
  playlists?: PlaylistOption[];
  onPlay: () => void;
  onLike?: () => void;
  onPlayNext?: () => void;
  onAddToQueue?: () => void;
  onAddToPlaylist?: (playlistId: string) => void;
  onBrowsePlaylists?: () => void;
  onRemove?: () => void;
  /**
   * Show the key and tempo columns. Defaults to what the connected library can
   * answer; set explicitly only to override that, as a story does.
   */
  showAnalysis?: boolean;
};

/**
 * The track table's column widths, shared by the header and the rows.
 *
 * They are explicit rather than flexed for the same reason the Slint version
 * makes them explicit: with `flex` the leftover space is distributed on top of
 * each cell's intrinsic width, so a long title in one row pushes its columns
 * out of line with every other row.
 */
const COLS = {
  number: "w-[30px] shrink-0 text-right",
  title: "min-w-0 flex-[45]",
  artist: "hidden min-w-0 flex-[28] md:block",
  album: "hidden min-w-0 flex-[27] lg:block",
  key: "w-10 shrink-0 text-right",
  bpm: "w-11 shrink-0 text-right",
  time: "w-11 shrink-0 text-right",
  actions: "w-[64px] shrink-0",
};

export const TrackListHeader = ({
  className,
  showAnalysis,
}: {
  className?: string;
  /**
   * Whether the library can report a key and a tempo at all. Only a
   * music-player daemon analyses its own tracks — against Subsonic or Jellyfin
   * the columns are hidden rather than shown blank for every row.
   */
  showAnalysis?: boolean;
}) => {
  const fromLibrary = useLibraryAnalysis();
  const show = showAnalysis ?? fromLibrary;
  return (
  <div
    className={cn(
      "flex h-[30px] items-center gap-3 border-b border-line px-3",
      "text-[10px] tracking-[1px] text-muted",
      className
    )}
  >
    <span className={cn(COLS.number, "font-mono tracking-normal")}>#</span>
    <span className={COLS.title}>TITLE</span>
    <span className={COLS.artist}>ARTIST</span>
    <span className={COLS.album}>ALBUM</span>
    {show && <span className={COLS.key}>KEY</span>}
    {show && <span className={COLS.bpm}>BPM</span>}
    <span className={COLS.time}>TIME</span>
    <span className={COLS.actions} />
  </div>
  );
};

/**
 * The desktop's `TrackRow`. Clicking anywhere plays; the heart and the "…"
 * menu are dim until the row is hovered.
 *
 * The row is a `div` with a click handler rather than a `<button>` so the
 * artist and album cells can stay links — a button cannot contain an anchor,
 * and those two links are how the desktop's "Go to album" is reached by mouse.
 */
const TrackRow = ({
  track,
  index,
  useTrackNumber,
  current,
  playlists = [],
  onPlay,
  onLike,
  onPlayNext,
  onAddToQueue,
  onAddToPlaylist,
  onBrowsePlaylists,
  onRemove,
  showAnalysis,
}: TrackRowProps) => {
  const navigate = useNavigate();
  const fromLibrary = useLibraryAnalysis();
  const show = showAnalysis ?? fromLibrary;
  const keyColor = keyColorFor(track.key);
  const hasMenu =
    onPlayNext || onAddToQueue || track.albumId || onAddToPlaylist || onRemove;

  return (
    <div
      role="button"
      tabIndex={0}
      // Labelled explicitly: without it the accessible name is the whole row
      // read out as one run — number, title, artist, album, duration.
      aria-label={`Play ${track.title}`}
      onClick={onPlay}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onPlay();
        }
      }}
      className={cn(
        "group/row relative flex h-[42px] cursor-pointer items-center gap-3 rounded-control px-3",
        current ? "bg-selected" : "hover:bg-hover"
      )}
    >
      {/* The key as a colour on the row's left edge, the way a DJ tool shows
          it: finding what mixes is a matter of spotting neighbouring colours,
          which is faster than reading two-character labels down a column. */}
      {show && keyColor && (
        <span
          aria-hidden="true"
          data-testid="key-marker"
          style={{ backgroundColor: keyColor }}
          className="absolute inset-y-[6px] left-0 w-[3px] rounded-full"
        />
      )}
      <span className={cn(COLS.number, "font-mono text-xs text-muted")}>
        {current ? (
          <Icons.music size={13} className="ml-auto text-accent" />
        ) : useTrackNumber ? (
          (track.trackNumber ?? index + 1)
        ) : (
          index + 1
        )}
      </span>

      <span
        className={cn(
          "truncate text-[13px]",
          COLS.title,
          current ? "font-semibold text-accent" : "text-fg"
        )}
      >
        {track.title}
      </span>

      <span className={cn(COLS.artist, "truncate text-xs text-dim")}>
        {track.artistId ? (
          <Link
            to={`/artists/${track.artistId}`}
            onClick={(event) => event.stopPropagation()}
            className="hover:text-fg hover:underline"
          >
            {track.artist}
          </Link>
        ) : (
          track.artist
        )}
      </span>

      <span className={cn(COLS.album, "truncate text-xs text-dim")}>
        {track.albumId ? (
          <Link
            to={`/albums/${track.albumId}`}
            onClick={(event) => event.stopPropagation()}
            className="hover:text-fg hover:underline"
          >
            {track.album}
          </Link>
        ) : (
          track.album
        )}
      </span>

      {show && (
        // Nothing at all when unknown: a key is either known or it is not, and
        // a placeholder here invites reading it as a value.
        <span
          className={cn(COLS.key, "font-mono text-xs font-semibold")}
          style={keyColor ? { color: keyColor } : undefined}
        >
          {track.key ?? ""}
        </span>
      )}
      {show && (
        // A dash here, unlike the key: a blank in a numeric column reads as
        // something having gone wrong.
        <span className={cn(COLS.bpm, "font-mono text-xs text-muted")}>
          {track.bpm ? Math.round(track.bpm) : "-"}
        </span>
      )}

      <span className={cn(COLS.time, "font-mono text-xs text-muted")}>
        {track.duration}
      </span>

      <span
        className={cn(COLS.actions, "flex items-center justify-end gap-[6px]")}
        onClick={(event) => event.stopPropagation()}
      >
        {onLike && (
          <LikeButton
            liked={track.liked}
            iconSize={16}
            size={26}
            subtle
            onClick={onLike}
          />
        )}
        {hasMenu && (
          <ContextMenu
            trigger={
              <IconButton
                icon={Icons.ellipsis}
                iconSize={15}
                size={26}
                aria-label={`More actions for ${track.title}`}
                className="opacity-35 transition-opacity group-hover/row:opacity-100"
              />
            }
          >
            {onPlayNext && (
              <ContextMenuItem
                icon={Icons.play}
                label="Play next"
                onClick={onPlayNext}
              />
            )}
            {onAddToQueue && (
              <ContextMenuItem
                icon={Icons.listMusic}
                label="Add to queue"
                onClick={onAddToQueue}
              />
            )}
            {track.albumId && (
              <ContextMenuItem
                icon={Icons.disc}
                label="Go to album"
                onClick={() => navigate(`/albums/${track.albumId}`)}
              />
            )}
            {onAddToPlaylist && playlists.length > 0 && (
              <>
                <ContextMenuSeparator />
                <ContextMenuLabel>ADD TO PLAYLIST</ContextMenuLabel>
                {playlists.slice(0, 3).map((playlist) => (
                  <ContextMenuItem
                    key={playlist.id}
                    icon={Icons.playlist}
                    label={playlist.name}
                    onClick={() => onAddToPlaylist(playlist.id)}
                  />
                ))}
                {playlists.length > 3 && onBrowsePlaylists && (
                  <ContextMenuItem
                    icon={Icons.ellipsis}
                    label="Browse all playlists…"
                    onClick={onBrowsePlaylists}
                  />
                )}
              </>
            )}
            {onRemove && (
              <>
                <ContextMenuSeparator />
                <ContextMenuItem
                  icon={Icons.trash}
                  label="Remove from playlist"
                  danger
                  onClick={onRemove}
                />
              </>
            )}
          </ContextMenu>
        )}
      </span>
    </div>
  );
};

export default TrackRow;
