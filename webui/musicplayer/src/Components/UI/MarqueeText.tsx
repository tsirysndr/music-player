import { useLayoutEffect, useRef, useState } from "react";
import cn from "./cn";

export type MarqueeTextProps = {
  text: string;
  /** Scroll speed in px/s — the desktop's 26. */
  speed?: number;
  /** Pause at each end, in ms. */
  dwell?: number;
  className?: string;
};

/**
 * One line that scrolls itself when it does not fit, then slides back.
 *
 * Text that fits is left completely alone; only an overlong title — an ICY
 * `StreamTitle` is routinely longer than the player bar — ever moves. Ported
 * from the desktop's `MarqueeText`, including the dwell at each end.
 */
const MarqueeText = ({
  text,
  speed = 26,
  dwell = 1800,
  className,
}: MarqueeTextProps) => {
  const viewport = useRef<HTMLDivElement>(null);
  const label = useRef<HTMLSpanElement>(null);
  const [overflow, setOverflow] = useState(0);
  const [atEnd, setAtEnd] = useState(false);

  // Measured after layout, and again whenever the box resizes — the player bar
  // is elastic, so a title that fits at one window width will not at another.
  useLayoutEffect(() => {
    const box = viewport.current;
    const span = label.current;
    if (!box || !span) return;
    const measure = () =>
      setOverflow(Math.max(0, span.scrollWidth - box.clientWidth));
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(box);
    return () => observer.disconnect();
  }, [text]);

  // A new song restarts the run from the beginning.
  useLayoutEffect(() => setAtEnd(false), [text]);

  const travel = overflow > 0 ? (overflow / speed) * 1000 : 0;

  useLayoutEffect(() => {
    if (overflow <= 0) return;
    const timer = window.setTimeout(
      () => setAtEnd((end) => !end),
      travel + dwell
    );
    return () => window.clearTimeout(timer);
  }, [atEnd, overflow, travel, dwell]);

  return (
    <div ref={viewport} className={cn("w-full overflow-hidden", className)}>
      <span
        ref={label}
        style={{
          transform: `translateX(${atEnd ? -overflow : 0}px)`,
          transitionDuration: `${travel}ms`,
        }}
        className="block w-max max-w-none whitespace-nowrap transition-transform ease-in-out"
      >
        {text}
      </span>
    </div>
  );
};

export default MarqueeText;
