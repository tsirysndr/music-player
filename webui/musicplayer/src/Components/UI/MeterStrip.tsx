import cn from "./cn";

export type MeterStripProps = {
  /** 0..1 */
  level: number;
  segments?: number;
  className?: string;
};

/**
 * The Mixxx-style segmented LED strip from the desktop's VFD readout: the top
 * three segments are the "high" colour, the next three "mid", the rest "low",
 * so a peak reads at a glance without a number.
 */
const MeterStrip = ({ level, segments = 16, className }: MeterStripProps) => {
  const lit = (Number.isFinite(level) ? level : 0) * segments;
  return (
    // Decorative: the VFD's own readout carries the information a screen
    // reader needs, and 16 unlabelled cells would only be noise.
    <div
      aria-hidden="true"
      data-testid="meter-strip"
      className={cn("flex h-[5px] w-full gap-[2px]", className)}
    >
      {Array.from({ length: segments }, (_, i) => (
        <div
          key={i}
          className={cn(
            "flex-1 rounded-[1px]",
            i >= lit
              ? "bg-meter-off"
              : i >= 13
                ? "bg-meter-high"
                : i >= 10
                  ? "bg-meter-mid"
                  : "bg-meter-low"
          )}
        />
      ))}
    </div>
  );
};

export default MeterStrip;
