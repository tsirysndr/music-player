import { Popover } from "@heroui/react";
import type { ReactNode } from "react";
import cn from "./cn";
import type { IconComponent } from "./icons";

export type ContextMenuProps = {
  /** The element that opens the menu — usually an `IconButton`. */
  trigger: ReactNode;
  width?: number;
  placement?: "bottom start" | "bottom end" | "top start" | "top end";
  className?: string;
  children: ReactNode;
};

/**
 * The popup the desktop hangs off every "…" button: a raised panel with a
 * hairline border and a deep shadow.
 *
 * HeroUI's popover (React Aria) supplies the positioning, the outside-click
 * dismiss and the Escape handling. Everything visible is ours, so it matches
 * the Slint `PopupWindow` rather than HeroUI's default surface.
 */
export const ContextMenu = ({
  trigger,
  width = 220,
  placement = "bottom end",
  className,
  children,
}: ContextMenuProps) => (
  <Popover>
    <Popover.Trigger>{trigger}</Popover.Trigger>
    <Popover.Content
      placement={placement}
      offset={6}
      style={{ width }}
      className={cn(
        // 4px of padding, not 6 — the rows carry their own, and the extra was
        // reading as a border of empty space around a short menu.
        "z-50 rounded-[10px] border border-line bg-raised p-1 shadow-[0_10px_30px_#000000aa]",
        "outline-none",
        className
      )}
    >
      <Popover.Dialog className="flex flex-col outline-none">
        {children}
      </Popover.Dialog>
    </Popover.Content>
  </Popover>
);

export type ContextMenuItemProps = {
  icon?: IconComponent;
  label: string;
  danger?: boolean;
  onClick?: () => void;
};

/** The desktop's `CtxMenuItem`: a 30px row with a dim leading glyph. */
export const ContextMenuItem = ({
  icon: Icon,
  label,
  danger,
  onClick,
}: ContextMenuItemProps) => (
  <button
    type="button"
    onClick={onClick}
    className="flex h-[30px] w-full items-center gap-[10px] px-2 text-left transition-colors hover:bg-hover"
  >
    {Icon && (
      <Icon
        size={13}
        className={cn("shrink-0", danger ? "text-meter-high" : "text-dim")}
      />
    )}
    <span
      className={cn(
        "truncate text-xs",
        danger ? "text-meter-high" : "text-fg"
      )}
    >
      {label}
    </span>
  </button>
);

/** The hairline the desktop draws between groups of menu items. */
export const ContextMenuSeparator = () => (
  <div className="my-[3px] h-px bg-line" />
);

/** A tracked-out caption above a group, e.g. "ADD TO PLAYLIST". */
export const ContextMenuLabel = ({ children }: { children: ReactNode }) => (
  <div className="flex h-5 items-center px-2 text-[9px] tracking-[1.5px] text-muted">
    {children}
  </div>
);

export default ContextMenu;
