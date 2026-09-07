import { forwardRef, type ButtonHTMLAttributes } from "react";
import cn from "./cn";
import type { IconComponent } from "./icons";

export type IconButtonProps = Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "children"
> & {
  icon: IconComponent;
  /** Glyph size in px. The desktop's default is 16. */
  iconSize?: number;
  /** Draw the glyph in the accent colour — the desktop's "on" state. */
  accented?: boolean;
  /** Button box in px. The desktop's default is 34. */
  size?: number;
};

/**
 * `IconButton` from `desktop/ui/components.slint`: a circular hover target
 * with a stroke glyph that goes from dim to full text colour on hover, or
 * sits in the accent colour when the thing it toggles is on.
 */
const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  (
    { icon: Icon, iconSize = 16, accented, size = 34, className, ...props },
    ref
  ) => (
    <button
      ref={ref}
      type="button"
      style={{ width: size, height: size }}
      className={cn(
        "group inline-flex shrink-0 items-center justify-center rounded-full",
        "transition-colors hover:bg-hover disabled:pointer-events-none disabled:opacity-40",
        className
      )}
      {...props}
    >
      <Icon
        size={iconSize}
        className={cn(
          "transition-colors",
          accented ? "text-accent" : "text-dim group-hover:text-fg"
        )}
      />
    </button>
  )
);

IconButton.displayName = "IconButton";

export default IconButton;
