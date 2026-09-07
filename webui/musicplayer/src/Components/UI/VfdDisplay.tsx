import cn from "./cn";
import { Icons } from "./icons";
import MeterStrip from "./MeterStrip";

export type VfdDisplayProps = {
  timeText?: string;
  infoText?: string;
  playing?: boolean;
  stopped?: boolean;
  vuLeft?: number;
  vuRight?: number;
  className?: string;
};

/**
 * The jetAudio-style readout that sits at the right of the desktop player bar:
 * a phosphor-coloured panel with the transport state, the elapsed time in the
 * mono face, two VU strips, and the stream's codec line underneath.
 */
const VfdDisplay = ({
  timeText = "00:00",
  infoText = "-- kbps  --.- kHz",
  playing,
  stopped = true,
  vuLeft = 0,
  vuRight = 0,
  className,
}: VfdDisplayProps) => (
  <div
    style={{ borderColor: "var(--display-glow)" }}
    className={cn(
      // The width is overridable: the player bar narrows it between lg and xl
      // so the transport keeps its room.
      "flex h-[62px] w-[220px] shrink-0 flex-col justify-between rounded-md border bg-display p-[10px]",
      className
    )}
  >
    <div className="flex items-center gap-2">
      <div className="flex w-3 shrink-0 items-center justify-center">
        {/* Stopped is a square — the tape-deck convention the desktop keeps. */}
        {stopped ? (
          <span className="size-2 bg-display-text" />
        ) : playing ? (
          <Icons.play size={11} className="text-display-text" />
        ) : (
          <Icons.pause size={11} className="text-display-text" />
        )}
      </div>
      <span className="font-mono text-[22px] leading-none font-semibold text-display-text tabular-nums">
        {timeText}
      </span>
      <div className="ml-auto flex w-[70px] shrink-0 flex-col gap-1">
        <MeterStrip level={vuLeft} />
        <MeterStrip level={vuRight} />
      </div>
    </div>
    <div className="truncate text-center font-mono text-[9px] text-display-dim">
      {infoText}
    </div>
  </div>
);

export default VfdDisplay;
