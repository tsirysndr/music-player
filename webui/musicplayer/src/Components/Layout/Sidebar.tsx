import { useAtom, useAtomValue, useSetAtom } from "jotai";
import { useLocation } from "react-router-dom";
import { useGetConnectedServerQuery } from "../../Hooks/GraphQL";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSkin } from "../../Providers/SkinProvider";
import {
  serverConnectedAtom,
  serverSwitcherOpenAtom,
  sidebarOpenAtom,
} from "../../State";
import { Icons, SidebarItem, cn } from "../UI";
import { isActive, NAV } from "./navigation";

/**
 * The desktop client's sidebar: a wordmark, the library sections, the recent
 * playlists, the skin switcher and the connection status.
 *
 * It collapses to zero width rather than unmounting, so the animation matches
 * the desktop's `animate width` and the scroll position survives a toggle.
 */
const Sidebar = () => {
  const [open] = useAtom(sidebarOpenAtom);
  const { pathname } = useLocation();
  const { skinName, cycleSkin } = useSkin();
  const { recentPlaylists } = usePlaylist();
  const connected = useAtomValue(serverConnectedAtom);
  const openServerSwitcher = useSetAtom(serverSwitcherOpenAtom);
  // Which server the library is being *read* from. Nothing to do with where
  // the audio comes out — that is the "play on" picker, one row up.
  const { data: server } = useGetConnectedServerQuery();
  const provider = server?.connectedServer;

  return (
    <aside
      style={{ width: open ? 208 : 0 }}
      className={cn(
        "hidden shrink-0 overflow-hidden bg-sidebar transition-[width] duration-150 ease-out lg:block"
      )}
    >
      <div className="flex h-full w-[208px] flex-col p-3">
        {/* Sized to fit 208px on one line in Roboto Mono, which is wider per
            character than the face this started in. */}
        <div className="flex items-center gap-2 px-4 pb-[22px] pt-[10px]">
          <Icons.disc size={18} className="shrink-0 text-accent" />
          <span className="whitespace-nowrap text-[13px] font-bold tracking-[1.5px] text-fg">
            MUSIC PLAYER
          </span>
        </div>

        <nav className="flex flex-col gap-1">
          {NAV.map((entry) => (
            <SidebarItem
              key={entry.to}
              to={entry.to}
              icon={entry.icon}
              label={entry.label}
              active={isActive(pathname, entry)}
            />
          ))}
        </nav>

        {recentPlaylists.length > 0 && (
          <div className="scrollbar-skin mt-5 min-h-0 flex-1 overflow-y-auto">
            <div className="px-4 pb-2 text-[9px] tracking-[1.5px] text-muted">
              PLAYLISTS
            </div>
            <div className="flex flex-col gap-[2px]">
              {recentPlaylists.map((playlist) => (
                <SidebarItem
                  key={playlist.id}
                  to={`/playlists/${playlist.id}`}
                  icon={Icons.playlist}
                  label={playlist.name}
                  active={pathname === `/playlists/${playlist.id}`}
                />
              ))}
            </div>
          </div>
        )}
        {recentPlaylists.length === 0 && <div className="flex-1" />}

        <button
          type="button"
          onClick={cycleSkin}
          className="flex h-[52px] flex-col justify-center gap-[2px] rounded-control pl-4 pr-2 text-left hover:bg-hover"
        >
          <span className="text-[9px] tracking-[1.5px] text-muted">SKIN</span>
          <span className="flex items-center gap-2">
            <Icons.settings size={13} className="text-accent" />
            <span className="text-xs text-dim">{skinName}</span>
          </span>
        </button>

        {/* The status row switches the *source* — where the library is read
            from — which is what the desktop's does. Where the audio comes out
            is a separate question, and a separate picker. */}
        <button
          type="button"
          onClick={() => openServerSwitcher(true)}
          title="Choose which server the library is read from"
          className="mt-[6px] flex h-[30px] items-center gap-2 rounded-control pl-4 pr-2 text-left hover:bg-hover"
        >
          {/* Green when we can reach the daemon, red when we cannot — the
              desktop's rule. */}
          <span
            aria-hidden="true"
            className={cn(
              "size-2 shrink-0 rounded-full",
              connected ? "bg-meter-low" : "bg-meter-high"
            )}
          />
          <span className="truncate font-mono text-[10px] text-muted">
            {provider ? provider.url : "this device"}
          </span>
        </button>
      </div>
    </aside>
  );
};

export default Sidebar;
