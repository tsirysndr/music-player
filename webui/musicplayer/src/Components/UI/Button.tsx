import { forwardRef, type ButtonHTMLAttributes } from "react";
import cn from "./cn";
import type { IconComponent } from "./icons";

export type ButtonVariant = "accent" | "ghost" | "outline" | "danger";

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  icon?: IconComponent;
  /** Fills the width of its container — what the modal footers use on mobile. */
  block?: boolean;
};

const VARIANTS: Record<ButtonVariant, string> = {
  // The pill the desktop uses for "New playlist" and "Add server".
  accent: "bg-accent text-on-accent hover:bg-accent-hover font-bold",
  ghost: "text-dim hover:bg-hover hover:text-fg",
  outline: "border border-line text-fg hover:bg-hover",
  danger: "bg-meter-high text-white hover:opacity-90 font-bold",
};

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    { variant = "accent", icon: Icon, block, className, children, ...props },
    ref
  ) => (
    <button
      ref={ref}
      type="button"
      className={cn(
        "inline-flex h-[34px] items-center justify-center gap-[6px] rounded-full px-4 text-xs",
        "transition-colors disabled:pointer-events-none disabled:opacity-40",
        VARIANTS[variant],
        block && "w-full",
        className
      )}
      {...props}
    >
      {Icon && <Icon size={14} className="shrink-0" />}
      {children}
    </button>
  )
);

Button.displayName = "Button";

export default Button;
