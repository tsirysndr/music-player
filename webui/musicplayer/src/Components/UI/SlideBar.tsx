import { useCallback, useEffect, useRef, useState } from "react";
import cn from "./cn";

export type SlideBarProps = {
  /** 0..1 */
  progress: number;
  /** Track thickness in px; the desktop uses 5 for seek, 4 for settings rows. */
  barHeight?: number;
  disabled?: boolean;
  className?: string;
  "aria-label"?: string;
  onChange: (value: number) => void;
};

/**
 * `SlideBar` from the desktop client — click or drag anywhere on the track to
 * set the value, with a knob that only appears while the pointer is on it.
 *
 * The drag is tracked on `window` rather than on the element: releasing the
 * button outside the bar is the normal way to end a drag, and a listener bound
 * to the bar would never see that `pointerup` and would leave it stuck down.
 */
const SlideBar = ({
  progress,
  barHeight = 5,
  disabled,
  className,
  onChange,
  "aria-label": ariaLabel = "Seek",
}: SlideBarProps) => {
  const trackRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState(false);
  const clamped = Math.min(1, Math.max(0, Number.isFinite(progress) ? progress : 0));

  const valueAt = useCallback((clientX: number) => {
    const track = trackRef.current;
    if (!track) return 0;
    const { left, width } = track.getBoundingClientRect();
    if (width === 0) return 0;
    return Math.min(1, Math.max(0, (clientX - left) / width));
  }, []);

  useEffect(() => {
    if (!dragging) return;
    const move = (event: PointerEvent) => onChange(valueAt(event.clientX));
    const up = () => setDragging(false);
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
  }, [dragging, onChange, valueAt]);

  return (
    <div
      role="slider"
      tabIndex={disabled ? -1 : 0}
      aria-label={ariaLabel}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(clamped * 100)}
      aria-disabled={disabled || undefined}
      onPointerDown={(event) => {
        if (disabled) return;
        event.preventDefault();
        onChange(valueAt(event.clientX));
        setDragging(true);
      }}
      onKeyDown={(event) => {
        if (disabled) return;
        const step = event.shiftKey ? 0.1 : 0.02;
        if (event.key === "ArrowRight" || event.key === "ArrowUp") {
          event.preventDefault();
          onChange(Math.min(1, clamped + step));
        }
        if (event.key === "ArrowLeft" || event.key === "ArrowDown") {
          event.preventDefault();
          onChange(Math.max(0, clamped - step));
        }
      }}
      className={cn(
        "group relative flex h-5 w-full min-w-[60px] touch-none items-center",
        disabled ? "cursor-default opacity-50" : "cursor-pointer",
        className
      )}
    >
      <div
        ref={trackRef}
        style={{ height: barHeight }}
        className="relative w-full rounded-full bg-track"
      >
        <div
          style={{ width: `${clamped * 100}%` }}
          className="h-full rounded-full bg-accent"
        />
        <div
          style={{ left: `${clamped * 100}%` }}
          className={cn(
            "pointer-events-none absolute top-1/2 size-[11px] -translate-x-1/2 -translate-y-1/2",
            "rounded-full bg-fg opacity-0 transition-opacity duration-100",
            "group-hover:opacity-100",
            dragging && "opacity-100"
          )}
        />
      </div>
    </div>
  );
};

export default SlideBar;
