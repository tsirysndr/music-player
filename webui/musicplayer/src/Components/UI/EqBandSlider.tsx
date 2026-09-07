import { useEffect, useRef, useState } from "react";
import cn from "./cn";

export type EqBandSliderProps = {
  /** Gain in dB × 10, -240..240 — the engine's own unit. */
  gain: number;
  /** The band's centre frequency, already formatted ("125", "16k"). */
  freqLabel: string;
  disabled?: boolean;
  className?: string;
  /** The new gain, in dB × 10. */
  onChange: (gain: number) => void;
};

const MIN = -240;
const MAX = 240;
const RANGE = MAX - MIN;

/**
 * A vertical EQ band, ported from the desktop's `EqBandSlider`: the gain
 * readout above, the fader, and the band's frequency below.
 *
 * Double-clicking returns the band to flat, which is the only quick way back
 * once ten of them have been dragged around.
 */
const EqBandSlider = ({
  gain,
  freqLabel,
  disabled,
  className,
  onChange,
}: EqBandSliderProps) => {
  const trackRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState(false);
  const clamped = Math.min(MAX, Math.max(MIN, gain));
  const norm = (clamped - MIN) / RANGE;

  const valueAt = (clientY: number) => {
    const track = trackRef.current;
    if (!track) return clamped;
    const { top, height } = track.getBoundingClientRect();
    if (height === 0) return clamped;
    const fromBottom = 1 - (clientY - top) / height;
    return Math.round(Math.min(1, Math.max(0, fromBottom)) * RANGE) + MIN;
  };

  useEffect(() => {
    if (!dragging) return;
    const move = (event: PointerEvent) => onChange(valueAt(event.clientY));
    const up = () => setDragging(false);
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dragging, onChange]);

  return (
    <div
      className={cn(
        "flex min-w-0 flex-1 flex-col items-center gap-[6px]",
        disabled && "pointer-events-none opacity-35",
        className
      )}
    >
      <span
        className={cn(
          "font-mono text-[9px]",
          clamped === 0 ? "text-muted" : "text-accent"
        )}
      >
        {(clamped / 10).toFixed(1)}
      </span>

      <div
        role="slider"
        tabIndex={disabled ? -1 : 0}
        aria-label={`${freqLabel} Hz`}
        aria-valuemin={MIN / 10}
        aria-valuemax={MAX / 10}
        aria-valuenow={clamped / 10}
        aria-valuetext={`${(clamped / 10).toFixed(1)} dB`}
        onPointerDown={(event) => {
          event.preventDefault();
          onChange(valueAt(event.clientY));
          setDragging(true);
        }}
        onDoubleClick={() => onChange(0)}
        onKeyDown={(event) => {
          // 10 = 1 dB, so a plain arrow is one decibel and shift is five.
          const step = event.shiftKey ? 50 : 10;
          if (event.key === "ArrowUp") {
            event.preventDefault();
            onChange(Math.min(MAX, clamped + step));
          }
          if (event.key === "ArrowDown") {
            event.preventDefault();
            onChange(Math.max(MIN, clamped - step));
          }
        }}
        className="relative h-[110px] w-full cursor-ns-resize touch-none"
      >
        <div
          ref={trackRef}
          className="absolute left-1/2 top-0 h-full w-[5px] -translate-x-1/2 overflow-hidden rounded-full bg-track"
        >
          <div
            style={{ height: `${norm * 100}%` }}
            className="absolute bottom-0 w-full bg-accent"
          />
        </div>
        {/* The 0 dB line, so flat is findable without reading the number. */}
        <div className="absolute left-1/2 top-1/2 h-px w-[13px] -translate-x-1/2 bg-line" />
        <div
          style={{ top: `${(1 - norm) * 100}%` }}
          className="absolute left-1/2 h-2 w-[14px] -translate-x-1/2 -translate-y-1/2 rounded-[3px] border border-window bg-fg"
        />
      </div>

      <span className="font-mono text-[9px] text-muted">{freqLabel}</span>
    </div>
  );
};

export default EqBandSlider;
