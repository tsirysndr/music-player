import { useEffect, useRef, type ReactNode } from "react";
import { Artwork, Dialog, Icons, cn, type IconComponent } from "../UI";

export type PaletteKind =
  | "track"
  | "album"
  | "artist"
  | "playlist"
  | "extension"
  | "server"
  | "radio";

/** One row of the palette. */
export type PaletteEntry = {
  key: string;
  kind: PaletteKind;
  title: string;
  subtitle?: string;
  cover?: string | null;
  icon: IconComponent;
  /** What activating the row does — play it, or go to it. */
  run: () => void;
  /**
   * A control at the right of the row, for a kind whose useful action is not
   * "open it" — an extension's switch, say. Rendered outside the row's own
   * button, because a button cannot contain another one.
   */
  action?: ReactNode;
};

export type CommandPaletteProps = {
  open: boolean;
  query: string;
  entries: PaletteEntry[];
  /** Index into `entries`. */
  selected: number;
  /** A search is in flight; the list shows what it has so far. */
  loading?: boolean;
  onQueryChange: (query: string) => void;
  onSelect: (index: number) => void;
  onActivate: (entry: PaletteEntry) => void;
  onClose: () => void;
};

const KIND_LABEL: Record<PaletteKind, string> = {
  track: "track",
  album: "album",
  artist: "artist",
  playlist: "playlist",
  extension: "extension",
  server: "server",
  radio: "radio",
};

/**
 * The command palette — the desktop client's search overlay, as a modal.
 *
 * It is the one place that searches *everything*: the library, plus the
 * playlists, extensions, servers and internet radio the pages used to filter
 * for themselves. Opened
 * with `/` or ⌘K.
 *
 * Presentational: `CommandPaletteWithData` builds the entries.
 */
const CommandPalette = ({
  open,
  query,
  entries,
  selected,
  loading,
  onQueryChange,
  onSelect,
  onActivate,
  onClose,
}: CommandPaletteProps) => {
  const listRef = useRef<HTMLUListElement>(null);

  // Keep the highlighted row in view as the selection moves by keyboard.
  useEffect(() => {
    listRef.current
      ?.querySelector('[data-selected="true"]')
      ?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  if (!open) return null;

  const trimmed = query.trim();

  return (
    // The same modal shell every other overlay uses, so the palette cannot
    // drift onto a different surface: `bare` because it supplies its own
    // layout rather than the padded body.
    <Dialog
      isOpen
      onClose={onClose}
      aria-label="Search"
      placement="top"
      width={620}
      bare
      className="max-h-[70vh]"
    >
      <div className="flex min-h-0 flex-col">
          <div className="flex items-center gap-3 border-b border-line px-4 py-3">
            <Icons.search size={17} className="shrink-0 text-muted" />
            <input
              autoFocus
              value={query}
              aria-label="Search"
              placeholder="Search tracks, albums, artists, playlists, extensions…"
              onChange={(event) => onQueryChange(event.target.value)}
              onKeyDown={(event) => {
                if (entries.length === 0) return;
                if (event.key === "ArrowDown") {
                  event.preventDefault();
                  onSelect((selected + 1) % entries.length);
                } else if (event.key === "ArrowUp") {
                  event.preventDefault();
                  onSelect((selected - 1 + entries.length) % entries.length);
                } else if (event.key === "Enter") {
                  event.preventDefault();
                  const entry = entries[selected];
                  if (entry) onActivate(entry);
                }
              }}
              className="min-w-0 flex-1 bg-transparent text-sm text-fg placeholder:text-muted"
            />
            <kbd className="hidden h-[18px] items-center rounded border border-line bg-hover px-[6px] font-mono text-[10px] text-dim sm:flex">
              esc
            </kbd>
          </div>

          <ul
            ref={listRef}
            className="scrollbar-skin min-h-0 flex-1 overflow-y-auto p-2"
          >
            {trimmed === "" ? (
              <li className="px-3 py-8 text-center text-xs text-muted">
                Search your library, playlists and extensions
              </li>
            ) : entries.length === 0 ? (
              <li className="px-3 py-8 text-center text-xs text-muted">
                {loading ? "Searching…" : `Nothing matches “${trimmed}”`}
              </li>
            ) : (
              entries.map((entry, index) => (
                <li
                  key={entry.key}
                  data-selected={index === selected}
                  onMouseMove={() => onSelect(index)}
                  className={cn(
                    "flex h-[46px] items-center rounded-control pr-3",
                    index === selected ? "bg-selected" : "hover:bg-hover"
                  )}
                >
                  <button
                    type="button"
                    onClick={() => onActivate(entry)}
                    className="flex h-full min-w-0 flex-1 items-center gap-3 px-3 text-left"
                  >
                    {entry.cover ? (
                      <Artwork
                        src={entry.cover}
                        alt=""
                        fallbackIcon={entry.icon}
                        iconSize={15}
                        rounded={entry.kind === "artist" ? "full" : "square"}
                        className="size-7"
                      />
                    ) : (
                      <entry.icon
                        size={15}
                        className={cn(
                          "w-7 shrink-0",
                          index === selected ? "text-accent" : "text-dim"
                        )}
                      />
                    )}
                    <span className="flex min-w-0 flex-1 flex-col">
                      <span className="truncate text-[13px] text-fg">
                        {entry.title}
                      </span>
                      {entry.subtitle && (
                        <span className="truncate text-[11px] text-dim">
                          {entry.subtitle}
                        </span>
                      )}
                    </span>
                    <span className="shrink-0 font-mono text-[10px] text-muted">
                      {KIND_LABEL[entry.kind]}
                    </span>
                  </button>
                  {entry.action && (
                    <span className="ml-3 shrink-0">{entry.action}</span>
                  )}
                </li>
              ))
            )}
          </ul>
      </div>
    </Dialog>
  );
};

export default CommandPalette;
