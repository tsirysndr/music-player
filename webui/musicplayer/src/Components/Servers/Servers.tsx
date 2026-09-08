import { FC } from "react";
import { AppShell } from "../Layout";
import {
  Button,
  EmptyState,
  IconButton,
  Icons,
  SkeletonBox,
  type IconComponent,
} from "../UI";

/** A saved server, as the daemon reports it. */
export type ServerItem = {
  id: string;
  /** The provider-registry key: `subsonic`, `jellyfin`, `music-player`, … */
  kind: string;
  name: string;
  url: string;
  username?: string | null;
  /** The library screens are reading from this one. */
  connected: boolean;
};

/** One entry of the add-server form, straight from the daemon's registry. */
export type SourceKind = {
  kind: string;
  displayName: string;
  needsCredentials: boolean;
  defaultPort: number;
  /** A backend that always talks to one address; the form drops the field. */
  fixedUrl?: string | null;
};

export type ServersProps = {
  servers: ServerItem[];
  kinds: SourceKind[];
  loading?: boolean;
  error?: string;
  /** Connecting to this id right now. */
  busyId?: string;
  onAdd: () => void;
  onConnect: (server: ServerItem) => void;
  onDisconnect: () => void;
  onDelete: (server: ServerItem) => void;
};

const KIND_ICON: Record<string, IconComponent> = {
  jellyfin: Icons.jellyfin,
  subsonic: Icons.navidrome,
  navidrome: Icons.navidrome,
  "music-player": Icons.server,
  kodi: Icons.device,
  plex: Icons.disc,
  rocksky: Icons.navidrome,
};

/**
 * One saved server — the desktop's `ServerRow`: a 56px row with the kind's
 * glyph, the name over `Kind · url`, a delete button that stays out of the way
 * until hover, and a plug on the right.
 */
const ServerRow = ({
  server,
  kindLabel,
  busy,
  onConnect,
  onDisconnect,
  onDelete,
}: {
  server: ServerItem;
  kindLabel: string;
  busy: boolean;
  onConnect: () => void;
  onDisconnect: () => void;
  onDelete: () => void;
}) => {
  const Icon = KIND_ICON[server.kind] ?? Icons.server;
  return (
    <div className="group/row flex h-[56px] items-center gap-3 rounded-control bg-panel px-[14px] hover:bg-hover">
      <Icon size={20} className="shrink-0 text-accent" />
      <button
        type="button"
        onClick={onConnect}
        disabled={busy || server.connected}
        className="flex min-w-0 flex-1 flex-col items-start text-left"
      >
        <span className="flex w-full min-w-0 items-center gap-2">
          {/* A dot, not a filled row: the row is still a row, this one just
              happens to be the live one. */}
          {server.connected && (
            <span
              aria-hidden="true"
              className="size-2 shrink-0 rounded-full bg-meter-low"
            />
          )}
          <span className="truncate text-[13px] font-semibold text-fg">
            {server.name}
          </span>
        </span>
        <span className="w-full truncate text-[11px] text-dim">
          {`${kindLabel} · ${server.url}`}
        </span>
      </button>

      <IconButton
        icon={Icons.trash}
        iconSize={14}
        aria-label={`Forget ${server.name}`}
        className="opacity-35 transition-opacity group-hover/row:opacity-100"
        onClick={onDelete}
      />
      {busy ? (
        <span className="w-[76px] shrink-0 text-right font-mono text-[10px] text-muted">
          Connecting…
        </span>
      ) : server.connected ? (
        // Actionable rather than a label: the row you are reading from is
        // where you would look to stop reading from it.
        <Button variant="outline" onClick={onDisconnect}>
          Disconnect
        </Button>
      ) : (
        <IconButton
          icon={Icons.connect}
          iconSize={16}
          accented
          aria-label={`Connect to ${server.name}`}
          onClick={onConnect}
        />
      )}
    </div>
  );
};

const RowSkeleton = () => (
  <div className="flex h-[56px] items-center gap-3 rounded-control bg-panel px-[14px]">
    <SkeletonBox className="size-5 rounded-full" />
    <div className="flex flex-1 flex-col gap-[6px]">
      <SkeletonBox className="h-3 w-40" />
      <SkeletonBox className="h-[10px] w-56" />
    </div>
  </div>
);

/**
 * The Servers page, laid out as the desktop lays it out: "Add server" above
 * the list, then a row per saved server.
 *
 * Connecting is the whole interaction. There is no browse view to navigate
 * into — every library screen reads through whichever server is connected, so
 * the green dot on the row and the sidebar showing its address *is* the
 * feedback. Whatever is playing keeps playing across a switch.
 *
 * Presentational — `ServersWithData` supplies the state.
 */
const Servers: FC<ServersProps> = ({
  servers,
  kinds,
  loading,
  error,
  busyId,
  onAdd,
  onConnect,
  onDisconnect,
  onDelete,
}) => {
  const label = (kind: string) =>
    kinds.find((entry) => entry.kind === kind)?.displayName ?? kind;
  const connected = servers.find((server) => server.connected);

  return (
    <AppShell title="Servers">
      {/* Above the list, not below it: with a screen's worth of servers the
          button would be off the bottom, and adding one is the reason to come
          here with none. */}
      <div className="flex items-center gap-3 pb-4">
        <Button onClick={onAdd}>
          <Icons.circlePlus size={14} />
          Add server
        </Button>
        <p className="min-w-0 flex-1 truncate text-xs text-muted">
          {connected
            ? `Reading from ${connected.name}.`
            : "Reading from this machine's library."}
        </p>
        {connected && (
          <Button variant="outline" onClick={onDisconnect}>
            Use local library
          </Button>
        )}
      </div>

      {error && (
        <p className="pb-4 text-xs text-meter-high" role="alert">
          {error}
        </p>
      )}

      {loading ? (
        <div className="flex flex-col gap-2">
          {Array.from({ length: 3 }, (_, index) => (
            <RowSkeleton key={index} />
          ))}
        </div>
      ) : servers.length === 0 ? (
        <EmptyState
          icon={Icons.server}
          title="No servers yet"
          hint="Add a Subsonic/Navidrome or Jellyfin server, or another music-player daemon, and every library screen reads from it."
          action={<Button onClick={onAdd}>Add server</Button>}
        />
      ) : (
        <div className="flex flex-col gap-2">
          {servers.map((server) => (
            <ServerRow
              key={server.id}
              server={server}
              kindLabel={label(server.kind)}
              busy={server.id === busyId}
              onConnect={() => onConnect(server)}
              onDisconnect={onDisconnect}
              onDelete={() => onDelete(server)}
            />
          ))}
        </div>
      )}
    </AppShell>
  );
};

export default Servers;
