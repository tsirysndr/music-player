import cn from "./cn";

export type ToggleProps = {
  checked?: boolean;
  disabled?: boolean;
  label?: string;
  className?: string;
  onChange: (checked: boolean) => void;
};

/** The desktop's `Toggle`: a 40×22 accent-filled switch with a 16px thumb. */
const Toggle = ({
  checked,
  disabled,
  label,
  className,
  onChange,
}: ToggleProps) => (
  <button
    type="button"
    role="switch"
    aria-checked={!!checked}
    aria-label={label}
    disabled={disabled}
    onClick={() => onChange(!checked)}
    className={cn(
      "relative h-[22px] w-10 shrink-0 rounded-full transition-colors duration-150",
      checked ? "bg-accent" : "bg-track",
      disabled && "pointer-events-none opacity-40",
      className
    )}
  >
    <span
      className={cn(
        "absolute top-1/2 size-4 -translate-y-1/2 rounded-full transition-all duration-150",
        checked ? "left-[21px] bg-on-accent" : "left-[3px] bg-dim"
      )}
    />
  </button>
);

export default Toggle;
