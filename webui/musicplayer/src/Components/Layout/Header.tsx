import { useAtom, useSetAtom } from "jotai";
import type { ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { paletteOpenAtom, queueOpenAtom, sidebarOpenAtom } from "../../State";
import { IconButton, Icons, SearchButton } from "../UI";
import { titleForPath } from "./navigation";

export type HeaderProps = {
  /** Overrides the route-derived title — detail pages name their record. */
  title?: string;
  /** Page-specific buttons, placed left of the search box. */
  actions?: ReactNode;
  /** Show a back chevron, as the desktop's detail views do. */
  onBack?: () => void;
};

/** The desktop's 62px header: title, page actions, search, panel toggles. */
const Header = ({ title, actions, onBack }: HeaderProps) => {
  const { pathname } = useLocation();
  const setPaletteOpen = useSetAtom(paletteOpenAtom);
  const [sidebarOpen, setSidebarOpen] = useAtom(sidebarOpenAtom);
  const [queueOpen, setQueueOpen] = useAtom(queueOpenAtom);

  return (
    <header className="flex h-[62px] shrink-0 items-center gap-3 bg-window px-4 lg:px-6">
      {onBack && (
        <IconButton
          icon={Icons.chevronLeft}
          iconSize={18}
          aria-label="Back"
          onClick={onBack}
        />
      )}
      <h1 className="min-w-0 flex-1 truncate text-lg font-bold text-fg lg:text-xl">
        {title ?? titleForPath(pathname)}
      </h1>

      {actions}

      <SearchButton
        className="hidden md:flex"
        onClick={() => setPaletteOpen(true)}
      />
      <IconButton
        icon={Icons.search}
        iconSize={17}
        aria-label="Search"
        className="md:hidden"
        onClick={() => setPaletteOpen(true)}
      />

      <div className="hidden items-center gap-[2px] lg:flex">
        <IconButton
          icon={sidebarOpen ? Icons.panelLeftFilled : Icons.panelLeft}
          iconSize={16}
          accented={sidebarOpen}
          aria-label="Toggle sidebar"
          onClick={() => setSidebarOpen((open) => !open)}
        />
        <IconButton
          icon={queueOpen ? Icons.panelRightFilled : Icons.panelRight}
          iconSize={16}
          accented={queueOpen}
          aria-label="Toggle queue"
          onClick={() => setQueueOpen((open) => !open)}
        />
      </div>
    </header>
  );
};

export default Header;
