import { useEffect, useRef } from "react";
import cn from "./cn";

export type EqualizerBarsProps = {
  /** 0..1, the left channel as the meter sees it. */
  left: number;
  right: number;
  /** Paused bars settle to the floor rather than freezing mid-bounce. */
  playing?: boolean;
  bars?: number;
  className?: string;
};

/**
 * The height each bar is heading for, given the level and the moment.
 *
 * The daemon measures four numbers — left, right, and each again below 200 Hz —
 * not a spectrum, so this is a *shaped* response rather than a real FFT. Two
 * things make it read as a spectrum anyway, and both are about how music
 * behaves rather than about making it look busy:
 *
 * Bars lean on the channel nearest them, so the display is stereo: a hard-panned
 * hi-hat lifts one side and not the other, which a mono meter cannot show.
 *
 * And low bars are steadier than high ones. Bass is sustained and treble is
 * transient, so a spectrum's left side moves slowly and its right side flickers;
 * without that, uniform bars read instantly as decoration.
 *
 * Exported for its own sake: it is the whole behaviour, and testing it through
 * a canvas would test the canvas.
 */
export const barTargets = (
  left: number,
  right: number,
  count: number,
  time: number
): number[] => {
  const safe = (value: number) => (Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0);
  const l = safe(left);
  const r = safe(right);

  return Array.from({ length: count }, (_, i) => {
    const position = count === 1 ? 0 : i / (count - 1);
    // 0 at the edges, 1 in the middle — how far this bar is from "its" channel.
    const blend = 1 - Math.abs(position - 0.5) * 2;
    const channel = position < 0.5 ? l + (r - l) * blend : r + (l - r) * blend;

    // Higher bars sit lower and move more, as a real spectrum does: energy
    // falls off with frequency, so a flat display would be wrong as well as
    // dull.
    const tilt = 1 - position * 0.45;
    const flicker =
      1 +
      0.22 *
        position *
        Math.sin(time * (5 + i * 1.7) + i * 2.399);

    return Math.max(0, Math.min(1, channel * tilt * flicker));
  });
};

/**
 * The theme's accent, as canvas can use it.
 *
 * A plain hex custom property, so it has to be read from the computed style
 * rather than written into canvas as `var(...)` — canvas resolves no CSS.
 * Cached and refreshed rarely: `getComputedStyle` forces a style
 * recalculation, and doing that sixty times a second to read one value that
 * changes when the user picks a different skin is waste.
 *
 * One flat colour, not a gradient: the bars are already distinguished by
 * height, and a vertical ramp on top of that reads as a second, competing
 * signal rather than as reinforcement.
 */
const readColor = (() => {
  let cached = "#ff2d95";
  let readAt = 0;

  return (now: number) => {
    if (now - readAt < 1000) return cached;
    readAt = now;
    cached =
      getComputedStyle(document.documentElement)
        .getPropertyValue("--accent")
        .trim() || cached;
    return cached;
  };
})();

/** How tall a floating peak cap is. */
const CAP_HEIGHT = 3;
/** How far a cap floats above a bar it is still resting on. */
const CAP_GAP = 3;

/**
 * A bar with rounded top corners and square feet.
 *
 * Square at the bottom because the bars sit on an edge — rounding there would
 * leave a visible gap under every one of them.
 */
const roundedTop = (
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number
) => {
  const r = Math.min(radius, width / 2, height);
  ctx.beginPath();
  ctx.moveTo(x, y + height);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.lineTo(x + width - r, y);
  ctx.quadraticCurveTo(x + width, y, x + width, y + r);
  ctx.lineTo(x + width, y + height);
  ctx.closePath();
  ctx.fill();
};

/** Fast up, slow down — the ballistics that make bars look like sound. */
const ease = (current: number, target: number) =>
  target > current
    ? current + (target - current) * 0.55
    : current + (target - current) * 0.12;

/**
 * Animated equalizer bars for the full-screen player.
 *
 * Driven by the daemon's `levels` subscription, so it moves with the audio
 * actually leaving the output — including on a remote server or a cast device,
 * where the browser has no audio element to analyse and a Web Audio
 * `AnalyserNode` would have nothing to listen to.
 */
const EqualizerBars = ({
  left,
  right,
  playing = true,
  bars = 96,
  className,
}: EqualizerBarsProps) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  // Read inside the animation frame rather than closed over, so the loop is
  // started once and not restarted on every level update twenty times a second.
  const levels = useRef({ left, right, playing });
  levels.current = { left, right, playing };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const heights = new Array(bars).fill(0);
    const peaks = new Array(bars).fill(0);
    let frame = 0;
    const started = performance.now();

    const draw = (now: number) => {
      frame = requestAnimationFrame(draw);

      const ratio = window.devicePixelRatio || 1;
      const width = canvas.clientWidth;
      const height = canvas.clientHeight;
      if (width === 0 || height === 0) return;
      if (canvas.width !== width * ratio || canvas.height !== height * ratio) {
        canvas.width = width * ratio;
        canvas.height = height * ratio;
      }
      ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
      ctx.clearRect(0, 0, width, height);

      const { left: l, right: r, playing: on } = levels.current;
      const targets = on
        ? barTargets(l, r, bars, (now - started) / 1000)
        : new Array(bars).fill(0);

      const slot = width / bars;
      // Nearly touching: with this many bars a wide gap reads as a row of
      // separate objects rather than as one moving surface.
      const barWidth = Math.max(1, slot * 0.88);
      const offset = (slot - barWidth) / 2;
      const radius = Math.min(barWidth / 2, 2);

      const color = readColor(now);
      ctx.fillStyle = color;

      for (let i = 0; i < bars; i++) {
        heights[i] = ease(heights[i], targets[i]);
        // Peaks fall at a constant rate rather than proportionally, so a cap
        // left by a loud hit drifts down visibly instead of hanging.
        peaks[i] = Math.max(heights[i], peaks[i] - 0.012);

        const barHeight = Math.max(2, heights[i] * height);
        roundedTop(ctx, offset + i * slot, height - barHeight, barWidth, barHeight, radius);
      }

      // Caps float clear of the bar rather than sitting on it, so a transient
      // that has already decayed is still visible as the level it reached.
      ctx.globalAlpha = 0.55;
      for (let i = 0; i < bars; i++) {
        if (peaks[i] <= 0.02) continue;
        const gap = peaks[i] - heights[i] > 0.02 ? 0 : CAP_GAP;
        const y = Math.max(0, Math.min(height - CAP_HEIGHT, height - peaks[i] * height - gap));
        roundedTop(ctx, offset + i * slot, y, barWidth, CAP_HEIGHT, radius);
      }
      ctx.globalAlpha = 1;
    };

    frame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(frame);
  }, [bars]);

  return (
    <canvas
      ref={canvasRef}
      // Decorative: the transport controls and the time readout already say
      // everything a screen reader needs about playback.
      aria-hidden="true"
      data-testid="equalizer-bars"
      className={cn("h-full w-full", className)}
    />
  );
};

export default EqualizerBars;
