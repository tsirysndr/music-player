import type { ButtonHTMLAttributes } from "react";
import cn from "./cn";
import { Icons } from "./icons";

export type PlayPauseButtonProps = Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "children"
> & {
  playing?: boolean;
  size?: number;
};

/**
 * `PlayPauseButton` from the desktop client: a filled accent disc that lifts
 * on hover. The glow is the desktop's `drop-shadow-blur: 8px → 14px`, which
 * is what gives the transport its centre of gravity.
 */
const PlayPauseButton = ({
  playing,
  size = 44,
  className,
  ...props
}: PlayPauseButtonProps) => {
  const Icon = playing ? Icons.pause : Icons.play;
  return (
    <button
      type="button"
      aria-label={playing ? "Pause" : "Play"}
      style={{
        width: size,
        height: size,
        // color-mix keeps the glow in the skin's accent hue whatever it is,
        // instead of hard-coding one shadow colour per skin.
        boxShadow: "0 0 8px color-mix(in srgb, var(--accent) 35%, transparent)",
      }}
      className={cn(
        "inline-flex shrink-0 items-center justify-center rounded-full",
        "bg-accent text-on-accent transition-all duration-150",
        "hover:bg-accent-hover hover:shadow-[0_0_14px_color-mix(in_srgb,var(--accent)_45%,transparent)]",
        "disabled:pointer-events-none disabled:opacity-40",
        className
      )}
      {...props}
    >
      {/* The play triangle is optically left-heavy, so it is nudged right —
          exactly the 1px the Slint version applies. */}
      <Icon
        size={Math.round(size * 0.41)}
        className={playing ? undefined : "translate-x-[1px]"}
      />
    </button>
  );
};

export default PlayPauseButton;
