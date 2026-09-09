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
  bars = 28,
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
      const barWidth = Math.max(1, slot * 0.62);
      const offset = (slot - barWidth) / 2;

      // One gradient across the whole display rather than a colour per bar:
      // the bars are already distinguished by height, and colouring by level as
      // well fights the shape instead of reinforcing it.
      const gradient = ctx.createLinearGradient(0, height, 0, 0);
      gradient.addColorStop(0, "rgb(var(--meter-low))");
      gradient.addColorStop(0.6, "rgb(var(--meter-mid))");
      gradient.addColorStop(1, "rgb(var(--meter-high))");
      ctx.fillStyle = gradient;

      for (let i = 0; i < bars; i++) {
        heights[i] = ease(heights[i], targets[i]);
        // Peaks fall at a constant rate rather than proportionally, so a cap
        // left by a loud hit drifts down visibly instead of hanging.
        peaks[i] = Math.max(heights[i], peaks[i] - 0.012);

        const barHeight = Math.max(2, heights[i] * height);
        ctx.fillRect(offset + i * slot, height - barHeight, barWidth, barHeight);
      }

      ctx.fillStyle = "rgb(var(--meter-high))";
      for (let i = 0; i < bars; i++) {
        if (peaks[i] <= 0.02) continue;
        const y = Math.min(height - 2, height - peaks[i] * height);
        ctx.fillRect(offset + i * slot, y, barWidth, 2);
      }
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
