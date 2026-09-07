import { Link } from "react-router-dom";
import cn from "./cn";
import type { IconComponent } from "./icons";

export type SidebarItemProps = {
  icon: IconComponent;
  label: string;
  active?: boolean;
  /** Renders as a router link when given, a button otherwise. */
  to?: string;
  onClick?: () => void;
};

/**
 * The desktop's `SidebarItem`: a 38px row that gains an accent rail down its
 * left edge and a weighted label when it is the current section.
 */
const SidebarItem = ({
  icon: Icon,
  label,
  active,
  to,
  onClick,
}: SidebarItemProps) => {
  const content = (
    <>
      <span
        className={cn(
          "absolute left-0 top-2 bottom-2 w-[3px] rounded-full bg-accent transition-opacity duration-150",
          active ? "opacity-100" : "opacity-0"
        )}
      />
      <Icon size={16} className={active ? "text-accent" : "text-dim"} />
      <span
        className={cn(
          "truncate text-[13px]",
          active ? "font-semibold text-fg" : "text-dim"
        )}
      >
        {label}
      </span>
    </>
  );

  const className = cn(
    "relative flex h-[38px] w-full items-center gap-3 rounded-control pl-4 pr-[10px]",
    "text-left transition-colors",
    active ? "bg-selected" : "hover:bg-hover"
  );

  if (to) {
    return (
      <Link to={to} onClick={onClick} className={className}>
        {content}
      </Link>
    );
  }
  return (
    <button type="button" onClick={onClick} className={className}>
      {content}
    </button>
  );
};

export default SidebarItem;
