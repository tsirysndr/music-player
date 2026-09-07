import { FC } from "react";
import { AppShell } from "../Layout";
import {
  Button,
  EmptyState,
  IconButton,
  Icons,
  SectionHeader,
  SkeletonBox,
  cn,
  type IconComponent,
} from "../UI";

/**
 * One row of the servers page. `kind` is the daemon's `app` field verbatim —
 * `music-player`, `subsonic`, `jellyfin`, `chromecast`, `xbmc` — because that
 * is what decides both the glyph and which connect mutation runs.
 */
export type ServerItem = {
  id: string;
  name: string;
  kind: string;
  /** `host:port`, when the daemon reported one. */
  address?: string;
  /** A cast target rather than a music source. */
  cast?: boolean;
};

export type ServersProps = {
  servers: ServerItem[];
  castDevices: ServerItem[];
  /** The id of whatever is currently playing our audio, if not this device. */
  connectedId?: string;
  loading?: boolean;
  error?: string;
  /** Connecting to this id right now. */
  busyId?: string;
  onConnect: (server: ServerItem) => void;
  onDisconnect: () => void;
  onRefresh: () => void;
};

const KIND_ICON: Record<string, IconComponent> = {
  jellyfin: Icons.jellyfin,
  subsonic: Icons.navidrome,
  navidrome: Icons.navidrome,
  chromecast: Icons.device,
  "music-player": Icons.server,
};

const KIND_LABEL: Record<string, string> = {
  jellyfin: "Jellyfin",
  subsonic: "Subsonic",
  navidrome: "Navidrome",
  chromecast: "Cast",
  xbmc: "Kodi",
  "music-player": "music-player",
};

/** The desktop's `ServerRow`: 56px, glyph, name over `kind · address`. */
const ServerRow = ({
  server,
  connected,
  busy,
  onConnect,
}: {
  server: ServerItem;
  connected: boolean;
  busy: boolean;
  onConnect: () => void;
}) => {
  const Icon = KIND_ICON[server.kind] ?? Icons.server;
  const label = KIND_LABEL[server.kind] ?? server.kind;
  return (
    <button
      type="button"
      onClick={onConnect}
      disabled={busy || connected}
      className={cn(
        "flex h-[56px] w-full items-center gap-3 rounded-control px-[14px] text-left",
        connected ? "bg-selected" : "bg-panel hover:bg-hover"
      )}
    >
      <Icon size={20} className={connected ? "text-accent" : "text-accent"} />
      <span className="min-w-0 flex-1">
        <span className="block truncate text-[13px] font-semibold text-fg">
          {server.name}
        </span>
        <span className="block truncate text-[11px] text-dim">
          {[label, server.address].filter(Boolean).join(" · ")}
        </span>
      </span>
      {busy ? (
        <span className="font-mono text-[10px] text-muted">Connecting…</span>
      ) : connected ? (
        <span className="text-[11px] font-semibold text-accent">Playing</span>
      ) : (
        // A span, not an `IconButton`: the whole row is already the button,
        // and nesting one inside another is invalid.
        <span
          aria-hidden="true"
          className="inline-flex size-[34px] shrink-0 items-center justify-center rounded-full"
        >
          <Icons.play size={15} className="text-accent" />
        </span>
      )}
    </button>
  );
};

const RowSkeleton = () => (
  <div className="flex h-[56px] items-center gap-3 rounded-control bg-panel px-[14px]">
    <SkeletonBox className="size-5 rounded-full" />
    <div className="flex flex-1 flex-col gap-[6px]">
      <SkeletonBox className="h-3 w-40" />
      <SkeletonBox className="h-[10px] w-24" />
    </div>
  </div>
);

/**
 * The Servers page — the desktop's fifth sidebar tab.
 *
 * It lists every place the daemon found on the network that can either hold
 * music (another music-player, a Subsonic/Navidrome or Jellyfin server) or
 * play it (a cast target), and lets you hand playback to one. "This device" is
 * the state you fall back to, so it is a button rather than a row.
 *
 * Presentational — `ServersWithData` supplies the state.
 */
const Servers: FC<ServersProps> = ({
  servers,
  castDevices,
  connectedId,
  loading,
  error,
  busyId,
  onConnect,
  onDisconnect,
  onRefresh,
}) => (
  <AppShell title="Servers">
    <div className="flex items-center gap-2 pb-4">
      <p className="min-w-0 flex-1 truncate text-xs text-muted">
        {connectedId
          ? "Playing on another device."
          : "Playing on this device."}
      </p>
      {connectedId && (
        <Button variant="outline" onClick={onDisconnect}>
          Play here instead
        </Button>
      )}
      <IconButton
        icon={Icons.refresh}
        iconSize={15}
        aria-label="Rescan the network"
        onClick={onRefresh}
      />
    </div>

    {error && (
      <p className="pb-4 text-xs text-meter-high" role="alert">
        {error}
      </p>
    )}

    {loading ? (
      <div className="flex flex-col gap-2">
        {Array.from({ length: 4 }, (_, index) => (
          <RowSkeleton key={index} />
        ))}
      </div>
    ) : servers.length === 0 && castDevices.length === 0 ? (
      <EmptyState
        icon={Icons.server}
        title="No servers found"
        hint="Other music-player instances, Subsonic/Navidrome and Jellyfin servers, and cast targets on this network show up here."
      />
    ) : (
      <div className="flex flex-col gap-6">
        {servers.length > 0 && (
          <div className="flex flex-col gap-2">
            <SectionHeader title="SERVERS" />
            {servers.map((server) => (
              <ServerRow
                key={server.id}
                server={server}
                connected={server.id === connectedId}
                busy={server.id === busyId}
                onConnect={() => onConnect(server)}
              />
            ))}
          </div>
        )}

        {castDevices.length > 0 && (
          <div className="flex flex-col gap-2">
            <SectionHeader title="CAST TARGETS" />
            {castDevices.map((device) => (
              <ServerRow
                key={device.id}
                server={device}
                connected={device.id === connectedId}
                busy={device.id === busyId}
                onConnect={() => onConnect(device)}
              />
            ))}
          </div>
        )}
      </div>
    )}
  </AppShell>
);

export default Servers;
