import { useEffect, useRef, type ReactNode } from "react";
import {
  Artwork,
  ContextMenu,
  Dialog,
  IconButton,
  Icons,
  cn,
  type IconComponent,
} from "../UI";

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
  /**
   * What the "…" menu offers for this row. Empty or absent means no menu:
   * what a result can *do* depends on where it came from, and a menu of
   * things that will not work is worse than none.
   */
  menu?: ReactNode;
  /** Where the row came from, shown when the two libraries are merged. */
  origin?: string;
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
  /**
   * Scoped to one kind. The server switcher is this overlay with `servers` —
   * same search, same keys, one kind of row.
   */
  scope?: "servers";
  /** Shown under the list; the switcher uses it for `C-n add`. */
  hint?: ReactNode;
  /** Ctrl-N in the server scope. */
  onAddServer?: () => void;
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
  scope,
  hint,
  onAddServer,
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
              placeholder={
                scope === "servers"
                  ? "Search servers, or type an address to connect…"
                  : "Search tracks, albums, artists, playlists, extensions…"
              }
              onChange={(event) => onQueryChange(event.target.value)}
              onKeyDown={(event) => {
                // Ctrl-N adds a server without leaving the flow, matching the
                // TUI and the Slint desktop. Before the empty-list guard: it
                // is exactly when nothing matched that you want it.
                if (
                  scope === "servers" &&
                  event.key.toLowerCase() === "n" &&
                  (event.ctrlKey || event.metaKey)
                ) {
                  event.preventDefault();
                  onAddServer?.();
                  return;
                }
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
            {/* A switcher lists everything before you type — that is what
                makes it a switcher rather than a search box. The library
                palette waits, because searching it costs a round trip. */}
            {trimmed === "" && scope !== "servers" ? (
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
                    {/* Where it came from, when both libraries are in the
                        list — otherwise the two "Fire Squad"s are
                        indistinguishable. */}
                    {entry.origin && (
                      <span className="shrink-0 truncate text-[10px] text-muted">
                        {entry.origin}
                      </span>
                    )}
                    <span className="shrink-0 font-mono text-[10px] text-muted">
                      {KIND_LABEL[entry.kind]}
                    </span>
                  </button>
                  {entry.action && (
                    <span className="ml-3 shrink-0">{entry.action}</span>
                  )}
                  {entry.menu && (
                    <span className="ml-1 shrink-0">
                      <ContextMenu
                        trigger={
                          <IconButton
                            icon={Icons.ellipsis}
                            iconSize={15}
                            aria-label={`More actions for ${entry.title}`}
                          />
                        }
                      >
                        {entry.menu}
                      </ContextMenu>
                    </span>
                  )}
                </li>
              ))
            )}
          </ul>
        {hint && (
          <div className="flex items-center gap-4 border-t border-line px-4 py-2 font-mono text-[10px] text-muted">
            {hint}
          </div>
        )}
      </div>
    </Dialog>
  );
};

export default CommandPalette;
