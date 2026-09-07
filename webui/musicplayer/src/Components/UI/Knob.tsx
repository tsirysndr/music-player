import { useCallback, useEffect, useRef, useState } from "react";
import cn from "./cn";

export type KnobProps = {
  label?: string;
  valueText?: string;
  /** 0..1 */
  norm: number;
  /** Where a double-click resets to. */
  defaultNorm?: number;
  /** Face diameter in px. */
  size?: number;
  /** Gap between the value readout and the face. */
  gap?: number;
  className?: string;
  "aria-label"?: string;
  onChange: (value: number) => void;
};

/** Pixels of vertical travel for a full sweep — the desktop's 140px. */
const TRAVEL = 140;

/**
 * The Mixxx-style rotary from the desktop client: drag up/down or scroll to
 * turn, double-click to reset. The indicator sweeps -135°…+135°.
 *
 * The indicator is a full-diameter bar that is only opaque over its top third,
 * so rotating it about its own centre pivots at the knob's centre without any
 * transform-origin arithmetic — the same trick the Slint version uses.
 */
const Knob = ({
  label,
  valueText,
  norm,
  defaultNorm = 0.5,
  size = 54,
  gap = 5,
  className,
  onChange,
  "aria-label": ariaLabel,
}: KnobProps) => {
  const clamped = Math.min(1, Math.max(0, Number.isFinite(norm) ? norm : 0));
  const [dragging, setDragging] = useState(false);
  // The value and pointer position the drag started from, so the knob tracks
  // total travel rather than accumulating per-event deltas (which drift).
  const origin = useRef({ y: 0, norm: 0 });

  const handleWheel = useCallback(
    (event: WheelEvent) => {
      event.preventDefault();
      onChange(Math.min(1, Math.max(0, clamped + (event.deltaY > 0 ? -0.04 : 0.04))));
    },
    [clamped, onChange]
  );

  const faceRef = useRef<HTMLDivElement>(null);

  // Bound by hand rather than via onWheel: React attaches wheel listeners
  // passively, and a passive listener may not preventDefault, so the page
  // would scroll while the knob turns.
  useEffect(() => {
    const face = faceRef.current;
    if (!face) return;
    face.addEventListener("wheel", handleWheel, { passive: false });
    return () => face.removeEventListener("wheel", handleWheel);
  }, [handleWheel]);

  useEffect(() => {
    if (!dragging) return;
    const move = (event: PointerEvent) => {
      const delta = (origin.current.y - event.clientY) / TRAVEL;
      onChange(Math.min(1, Math.max(0, origin.current.norm + delta)));
    };
    const up = () => setDragging(false);
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
  }, [dragging, onChange]);

  return (
    <div
      className={cn("flex select-none flex-col items-center", className)}
      style={{ gap, width: Math.max(size + 16, 72) }}
    >
      {valueText !== undefined && (
        <span className="font-mono text-[9px] text-accent">{valueText}</span>
      )}
      <div
        ref={faceRef}
        role="slider"
        tabIndex={0}
        aria-label={ariaLabel ?? label ?? "Knob"}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(clamped * 100)}
        onPointerDown={(event) => {
          event.preventDefault();
          origin.current = { y: event.clientY, norm: clamped };
          setDragging(true);
        }}
        onDoubleClick={() => onChange(defaultNorm)}
        onKeyDown={(event) => {
          const step = event.shiftKey ? 0.1 : 0.02;
          if (event.key === "ArrowUp" || event.key === "ArrowRight") {
            event.preventDefault();
            onChange(Math.min(1, clamped + step));
          }
          if (event.key === "ArrowDown" || event.key === "ArrowLeft") {
            event.preventDefault();
            onChange(Math.max(0, clamped - step));
          }
        }}
        style={{
          width: size,
          height: size,
          background:
            "linear-gradient(180deg, var(--panel-raised) 0%, var(--panel-bg) 100%)",
          borderColor: "var(--slider-track)",
          boxShadow: "0 2px 6px #00000066",
        }}
        className="relative cursor-ns-resize touch-none rounded-full border-2"
      >
        <div
          style={{
            height: size - 12,
            transform: `translate(-50%, -50%) rotate(${-135 + clamped * 270}deg)`,
            background:
              "linear-gradient(180deg, var(--accent) 0%, var(--accent) 36%, transparent 36%, transparent 100%)",
          }}
          className="absolute left-1/2 top-1/2 w-[3px]"
        />
        <div
          style={{ width: size * 0.28, height: size * 0.28 }}
          className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full border border-line bg-raised"
        />
      </div>
      {label ? (
        <span className="truncate text-[10px] text-dim">{label}</span>
      ) : null}
    </div>
  );
};

export default Knob;
