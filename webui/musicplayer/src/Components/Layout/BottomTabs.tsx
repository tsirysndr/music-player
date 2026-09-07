import { useAtom } from "jotai";
import { Link, useLocation } from "react-router-dom";
import { useSkin } from "../../Providers/SkinProvider";
import { mobileMenuOpenAtom } from "../../State";
import { Icons, cn } from "../UI";
import { isActive, NAV, PRIMARY_TABS } from "./navigation";

/**
 * The phone navigation. The sidebar is gone below `lg`, so the four most-used
 * sections become tabs and everything else moves into a sheet behind "More" —
 * which also carries the skin switcher the sidebar would otherwise own.
 */
const BottomTabs = () => {
  const { pathname } = useLocation();
  const [menuOpen, setMenuOpen] = useAtom(mobileMenuOpenAtom);
  const { skinName, cycleSkin } = useSkin();

  const primary = PRIMARY_TABS.map(
    (path) => NAV.find((entry) => entry.to === path)!
  );
  const overflow = NAV.filter((entry) => !PRIMARY_TABS.includes(entry.to));
  const overflowActive = overflow.some((entry) => isActive(pathname, entry));

  return (
    <>
      {menuOpen && (
        <>
          {/* Pointer affordance only — the More button toggles the sheet, so
              a keyboard user already has a way back out. */}
          <button
            type="button"
            aria-hidden="true"
            tabIndex={-1}
            onClick={() => setMenuOpen(false)}
            className="fixed inset-0 z-40 bg-black/60 lg:hidden"
          />
          <div className="fixed inset-x-0 bottom-[calc(56px+env(safe-area-inset-bottom))] z-40 rounded-t-2xl border-t border-line bg-panel p-3 lg:hidden">
            {overflow.map((entry) => (
              <Link
                key={entry.to}
                to={entry.to}
                onClick={() => setMenuOpen(false)}
                className={cn(
                  "flex h-12 items-center gap-3 rounded-control px-3",
                  isActive(pathname, entry) ? "bg-selected" : "hover:bg-hover"
                )}
              >
                <entry.icon
                  size={18}
                  className={
                    isActive(pathname, entry) ? "text-accent" : "text-dim"
                  }
                />
                <span className="text-sm text-fg">{entry.label}</span>
              </Link>
            ))}
            <button
              type="button"
              onClick={cycleSkin}
              className="flex h-12 w-full items-center gap-3 rounded-control px-3 text-left hover:bg-hover"
            >
              <Icons.settings size={18} className="text-accent" />
              <span className="flex-1 text-sm text-fg">Skin</span>
              <span className="text-xs text-dim">{skinName}</span>
            </button>
          </div>
        </>
      )}

      <nav className="fixed inset-x-0 bottom-0 z-40 flex h-14 border-t border-line bg-sidebar pb-[env(safe-area-inset-bottom)] lg:hidden">
        {primary.map((entry) => {
          const active = isActive(pathname, entry);
          return (
            <Link
              key={entry.to}
              to={entry.to}
              onClick={() => setMenuOpen(false)}
              className="flex flex-1 flex-col items-center justify-center gap-1"
            >
              <entry.icon
                size={20}
                className={active ? "text-accent" : "text-dim"}
              />
              <span
                className={cn(
                  "text-[10px]",
                  active ? "font-semibold text-accent" : "text-dim"
                )}
              >
                {entry.label}
              </span>
            </Link>
          );
        })}
        <button
          type="button"
          onClick={() => setMenuOpen((open) => !open)}
          aria-expanded={menuOpen}
          className="flex flex-1 flex-col items-center justify-center gap-1"
        >
          <Icons.ellipsis
            size={20}
            className={menuOpen || overflowActive ? "text-accent" : "text-dim"}
          />
          <span
            className={cn(
              "text-[10px]",
              menuOpen || overflowActive
                ? "font-semibold text-accent"
                : "text-dim"
            )}
          >
            More
          </span>
        </button>
      </nav>
    </>
  );
};

export default BottomTabs;
