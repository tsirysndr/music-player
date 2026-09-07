import cn from "./cn";
import SlideBar from "./SlideBar";

export type SettingRowProps = {
  label: string;
  /** The value as the user reads it — "+3.0 dB", "2 s", "L 20". */
  valueText: string;
  /** 0..1 */
  progress: number;
  disabled?: boolean;
  className?: string;
  onChange: (value: number) => void;
};

/**
 * The desktop's `SettingRow`: label, slider, value readout, on one line.
 *
 * The row deals in a 0..1 position and a string; converting to and from the
 * engine's units is the caller's job, because only the caller knows whether
 * the range is -24..24 dB or 0..15 seconds.
 */
const SettingRow = ({
  label,
  valueText,
  progress,
  disabled,
  className,
  onChange,
}: SettingRowProps) => (
  <div
    className={cn(
      "flex h-7 items-center gap-3",
      disabled && "pointer-events-none opacity-35",
      className
    )}
  >
    <span className="w-[130px] shrink-0 truncate text-xs text-dim">
      {label}
    </span>
    <SlideBar
      progress={progress}
      barHeight={4}
      aria-label={label}
      disabled={disabled}
      onChange={onChange}
    />
    <span className="w-[62px] shrink-0 text-right font-mono text-[11px] text-fg">
      {valueText}
    </span>
  </div>
);

export default SettingRow;
