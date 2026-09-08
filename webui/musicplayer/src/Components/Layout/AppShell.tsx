import { useAtom, useAtomValue, useSetAtom } from "jotai";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useVolume, VOLUME_STEP } from "../../Hooks/useVolume";
import { useSkin } from "../../Providers/SkinProvider";
import { nowPlayingAtom } from "../../State";
import {
  audioSettingsOpenAtom,
  fullPlayerOpenAtom,
  paletteOpenAtom,
  queueOpenAtom,
  sidebarOpenAtom,
} from "../../State";
import AudioSettingsWithData from "./AudioSettingsWithData";
import CommandPaletteWithData from "./CommandPaletteWithData";
import DeviceDialog from "./DeviceDialog";
import BottomTabs from "./BottomTabs";
import FullPlayerWithData from "./FullPlayerWithData";
import Header, { type HeaderProps } from "./Header";
import PlayerBarWithData from "./PlayerBarWithData";
import QueueDrawer from "./QueueDrawer";
import Sidebar from "./Sidebar";

export type AppShellProps = HeaderProps & {
  children: ReactNode;
  /** Drop the standard 24px content gutter — pages that scroll edge to edge. */
  bare?: boolean;
};

/**
 * The window, laid out exactly as the desktop client lays out its own:
 * sidebar | (header / content / player bar) | queue.
 *
 * Every page renders only its content and lets this supply the chrome, which
 * is what stops the player bar from remounting — and playback from stuttering
 * — on navigation.
 */
const AppShell = ({ children, bare, ...header }: AppShellProps) => {
  const { pathname } = useLocation();
  const navigate = useNavigate();
  const [sidebarOpen, setSidebarOpen] = useAtom(sidebarOpenAtom);
  const setQueueOpen = useSetAtom(queueOpenAtom);
  const [fullPlayerOpen, setFullPlayer] = useAtom(fullPlayerOpenAtom);
  const setPaletteOpen = useSetAtom(paletteOpenAtom);
  const setAudioOpen = useSetAtom(audioSettingsOpenAtom);
  const { cycleSkin } = useSkin();
  const { adjust: adjustVolume, toggleMute } = useVolume();
  const nowPlaying = useAtomValue(nowPlayingAtom);
  const [devicesOpen, setDevicesOpen] = useState(false);

  /**
   * The full player is a canvas for the current track's art and title, so with
   * nothing playing there is nothing to show. A station counts: it has a name
   * and usually a logo even before ICY metadata names the song.
   */
  const canGoFullscreen = !!nowPlaying?.title;

  /**
   * The desktop's global shortcuts, minus the ones a browser owns. Ignored
   * while a text field has focus, or typing "radio" into a filter would
   * navigate away mid-word.
   */
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typing =
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.isContentEditable;
      // ⌘K / Ctrl-K opens the palette from anywhere, including a text field —
      // it is the one shortcut that has to work while something has focus.
      if (event.key.toLowerCase() === "k" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        setPaletteOpen((open) => !open);
        return;
      }
      if (typing || event.metaKey || event.ctrlKey || event.altKey) return;

      switch (event.key) {
        case "/":
          event.preventDefault();
          setPaletteOpen(true);
          break;
        case "r":
          navigate("/radio");
          break;
        case "s":
          cycleSkin();
          break;
        case "q":
          setQueueOpen((open) => !open);
          break;
        case "b":
          sidebarBeforeFullPlayer.current = undefined;
          setSidebarOpen((open) => !open);
          break;
        case "e":
          setAudioOpen((open) => !open);
          break;
        case "f":
          // Does nothing with nothing playing, rather than opening an empty
          // canvas the user then has to dismiss.
          if (canGoFullscreen) setFullPlayer((open) => !open);
          break;
        // `=` and `_` are the unshifted faces of `+` and `-` on most layouts,
        // so both reach the same place.
        case "+":
        case "=":
          event.preventDefault();
          adjustVolume(VOLUME_STEP);
          break;
        case "-":
        case "_":
          event.preventDefault();
          adjustVolume(-VOLUME_STEP);
          break;
        case "m":
          toggleMute();
          break;
        case "Escape":
          setFullPlayer(false);
          break;
        default:
          break;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    navigate,
    cycleSkin,
    setQueueOpen,
    setSidebarOpen,
    setFullPlayer,
    setPaletteOpen,
    setAudioOpen,
    canGoFullscreen,
    adjustVolume,
    toggleMute,
  ]);

  /**
   * The full player wants the window; the sidebar is navigation for a page
   * that is no longer showing.
   *
   * What it was before is remembered rather than assumed, so someone who had
   * it collapsed does not get it back on Escape. The ref is only set while we
   * are the reason it is closed — reopening it by hand with `b` while the
   * canvas is up clears the claim, so closing the canvas leaves it alone.
   */
  const sidebarBeforeFullPlayer = useRef<boolean | undefined>(undefined);
  const fullPlayerWasOpen = useRef(false);
  useEffect(() => {
    const opened = fullPlayerOpen && !fullPlayerWasOpen.current;
    const closed = !fullPlayerOpen && fullPlayerWasOpen.current;
    fullPlayerWasOpen.current = fullPlayerOpen;

    // Acts on the edges, not the state: re-running on every sidebar change
    // would undo a `b` press the moment it happened.
    if (opened) {
      sidebarBeforeFullPlayer.current = sidebarOpen;
      if (sidebarOpen) setSidebarOpen(false);
    } else if (closed && sidebarBeforeFullPlayer.current !== undefined) {
      setSidebarOpen(sidebarBeforeFullPlayer.current);
      sidebarBeforeFullPlayer.current = undefined;
    }
  }, [fullPlayerOpen, sidebarOpen, setSidebarOpen]);

  // A phone has no room for both the sheet and the page; close it on
  // navigation so a tap on a result does not land behind the queue.
  useEffect(() => {
    if (window.matchMedia("(max-width: 1023px)").matches) {
      setQueueOpen(false);
    }
  }, [pathname, setQueueOpen]);

  return (
    <div className="flex h-dvh w-full overflow-hidden bg-window text-fg">
      <Sidebar onOpenDevices={() => setDevicesOpen(true)} />

      <div className="flex min-w-0 flex-1 flex-col">
        <Header {...header} />
        <main
          className={
            bare
              ? "scrollbar-skin min-h-0 flex-1 overflow-y-auto"
              : "scrollbar-skin min-h-0 flex-1 overflow-y-auto px-4 pb-6 lg:px-6"
          }
        >
          {children}
        </main>
        <PlayerBarWithData />
        {/* Clears the fixed bottom tab bar on small screens. */}
        <div className="h-14 shrink-0 lg:hidden" />
      </div>

      <QueueDrawer />
      <BottomTabs />
      <FullPlayerWithData />
      <CommandPaletteWithData />
      <AudioSettingsWithData />
      <DeviceDialog
        isOpen={devicesOpen}
        onClose={() => setDevicesOpen(false)}
      />
    </div>
  );
};

export default AppShell;
