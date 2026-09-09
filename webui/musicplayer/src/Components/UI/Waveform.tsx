import { useCallback, useRef } from "react";
import cn from "./cn";

export type WaveformProps = {
  /** Peak per bar, 0–255, left to right. Empty while unanalysed. */
  bars: number[];
  /** How far through, 0..1. */
  progress: number;
  /** Seconds, for turning a click into a position. */
  duration: number;
  onSeek?: (seconds: number) => void;
  className?: string;
};

/**
 * The position a click at `x` means, 0..1.
 *
 * Its own function because the arithmetic is the part worth being sure about —
 * an off-by-one on the bounding box seeks to the wrong place, which is a bug a
 * user notices immediately and a rendering test never would.
 */
export const positionAt = (x: number, boxLeft: number, boxWidth: number): number => {
  if (boxWidth <= 0) return 0;
  return Math.max(0, Math.min(1, (x - boxLeft) / boxWidth));
};

/**
 * The track's waveform, as a seek bar.
 *
 * Bars come from the daemon's analysis: real peaks from the decoded audio, so
 * the shape is the song — the quiet intro, the drop, the fade — and a listener
 * can aim at a part they remember rather than at a percentage.
 *
 * Drawn as divs rather than a canvas. There are a few hundred of them, they
 * only change when the playhead moves past one, and this way they inherit the
 * theme's colours and the browser handles hit-testing for the seek.
 */
const Waveform = ({
  bars,
  progress,
  duration,
  onSeek,
  className,
}: WaveformProps) => {
  const ref = useRef<HTMLDivElement>(null);

  const seek = useCallback(
    (clientX: number) => {
      if (!onSeek || !ref.current || duration <= 0) return;
      const box = ref.current.getBoundingClientRect();
      onSeek(positionAt(clientX, box.left, box.width) * duration);
    },
    [onSeek, duration]
  );

  if (bars.length === 0) {
    // Nothing analysed yet. A flat line rather than an empty space, so the
    // player's layout does not jump when the analysis arrives.
    return (
      <div
        data-testid="waveform-empty"
        className={cn("h-full w-full self-center", className)}
      >
        <div className="h-[2px] w-full rounded-full bg-line" />
      </div>
    );
  }

  const playedUpTo = Math.round(
    Math.max(0, Math.min(1, progress)) * bars.length
  );

  return (
    <div
      ref={ref}
      role={onSeek ? "slider" : undefined}
      tabIndex={onSeek ? 0 : undefined}
      aria-label={onSeek ? "Seek" : undefined}
      aria-valuemin={onSeek ? 0 : undefined}
      aria-valuemax={onSeek ? Math.round(duration) : undefined}
      aria-valuenow={onSeek ? Math.round(progress * duration) : undefined}
      data-testid="waveform"
      onClick={(event) => seek(event.clientX)}
      onKeyDown={(event) => {
        if (!onSeek || duration <= 0) return;
        // Five seconds a press, the same step the transport uses.
        const step =
          event.key === "ArrowRight" ? 5 : event.key === "ArrowLeft" ? -5 : 0;
        if (step === 0) return;
        event.preventDefault();
        onSeek(Math.max(0, Math.min(duration, progress * duration + step)));
      }}
      className={cn(
        "flex h-full w-full items-center gap-[1px]",
        onSeek && "cursor-pointer",
        className
      )}
    >
      {bars.map((bar, index) => (
        <div
          key={index}
          className={cn(
            "min-w-px flex-1 rounded-[1px] transition-colors",
            index < playedUpTo ? "bg-accent" : "bg-line"
          )}
          // Percent, so the waveform fills whatever height it is given.
          style={{ height: `${Math.max(3, (bar / 255) * 100)}%` }}
        />
      ))}
    </div>
  );
};

export default Waveform;
