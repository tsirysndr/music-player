import type { ReactNode } from "react";
import {
  Dialog as AriaDialog,
  Modal as AriaModal,
  ModalOverlay,
} from "react-aria-components";
import cn from "./cn";
import IconButton from "./IconButton";
import { Icons } from "./icons";

export type DialogProps = {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  /** Icon shown to the left of the title, as the desktop's modals do. */
  icon?: (typeof Icons)[keyof typeof Icons];
  /** Max width of the panel; the desktop's audio modal is 780. */
  width?: number;
  /** Clicking the backdrop dismisses. Off for destructive confirmations. */
  isDismissable?: boolean;
  footer?: ReactNode;
  className?: string;
  /** Sits the panel near the top, for a palette rather than a dialog. */
  placement?: "center" | "top";
  /** Replaces the padded body — a palette supplies its own layout. */
  bare?: boolean;
  children?: ReactNode;
  "aria-label"?: string;
};

/**
 * The modal shell every overlay in the desktop client shares: a dimmed
 * backdrop, a panel with a hairline border and a deep shadow, a title row and
 * an "esc to close" hint.
 *
 * Built on React Aria's *unstyled* primitives rather than HeroUI's modal
 * parts. HeroUI's carry their own surface colours from its default theme,
 * which fight the skin tokens and win often enough that a panel comes out in
 * the wrong palette. React Aria gives the same focus trapping, scroll locking
 * and Escape handling with none of the styling — which is what the desktop
 * gets from the window manager and the browser does not.
 */
const Dialog = ({
  isOpen,
  onClose,
  title,
  icon: Icon,
  width = 520,
  isDismissable = true,
  footer,
  className,
  placement = "center",
  bare,
  children,
  "aria-label": ariaLabel,
}: DialogProps) => (
  <ModalOverlay
    isOpen={isOpen}
    onOpenChange={(open) => {
      if (!open) onClose();
    }}
    isDismissable={isDismissable}
    className={cn(
      "fixed inset-0 z-50 flex justify-center bg-black/60 p-4 backdrop-blur-[2px]",
      placement === "top" ? "items-start pt-[12vh]" : "items-center"
    )}
  >
    <AriaModal
      style={{ maxWidth: width }}
      className={cn(
        "flex max-h-[calc(100dvh-4rem)] w-full flex-col overflow-hidden rounded-[14px]",
        "border border-line bg-panel text-fg shadow-[0_24px_60px_#000000aa]",
        "outline-none",
        className
      )}
    >
      <AriaDialog
        aria-label={ariaLabel ?? title}
        className="flex min-h-0 flex-col outline-none"
      >
        {title && (
          <div className="flex items-center gap-[10px] px-[22px] pb-2 pt-[22px]">
            {Icon && <Icon size={18} className="text-accent" />}
            <h2 className="flex-1 truncate text-base font-bold text-fg">
              {title}
            </h2>
            <span className="hidden font-mono text-[10px] text-muted sm:block">
              esc to close
            </span>
            <IconButton
              icon={Icons.close}
              iconSize={15}
              size={28}
              aria-label="Close"
              onClick={onClose}
            />
          </div>
        )}
        {bare ? (
          children
        ) : (
          <div className="scrollbar-skin min-h-0 flex-1 overflow-y-auto px-[22px] py-3">
            {children}
          </div>
        )}
        {footer && (
          <div className="flex items-center justify-end gap-2 border-t border-line px-[22px] py-4">
            {footer}
          </div>
        )}
      </AriaDialog>
    </AriaModal>
  </ModalOverlay>
);

export default Dialog;
