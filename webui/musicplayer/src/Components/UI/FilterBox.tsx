import { useId } from "react";
import cn from "./cn";
import { Icons } from "./icons";

export type FilterBoxProps = {
  value: string;
  placeholder?: string;
  className?: string;
  autoFocus?: boolean;
  "aria-label"?: string;
  onChange: (value: string) => void;
};

/**
 * The desktop's `RadioFilterBox`, used everywhere a list filters in place: a
 * 30px pill with a leading magnifier, an accent border while focused, and a
 * clear button that only exists once there is something to clear.
 */
const FilterBox = ({
  value,
  placeholder = "Filter…",
  className,
  autoFocus,
  onChange,
  "aria-label": ariaLabel,
}: FilterBoxProps) => {
  const id = useId();
  return (
    <div
      className={cn(
        "flex h-[30px] items-center gap-[7px] rounded-control border border-line bg-panel pl-[9px] pr-[6px]",
        "transition-colors focus-within:border-accent",
        className
      )}
    >
      <Icons.search size={13} className="shrink-0 text-muted" />
      <input
        id={id}
        type="text"
        value={value}
        autoFocus={autoFocus}
        placeholder={placeholder}
        aria-label={ariaLabel ?? placeholder}
        onChange={(event) => onChange(event.target.value)}
        className="min-w-0 flex-1 bg-transparent text-xs text-fg placeholder:text-muted"
      />
      {value !== "" && (
        <button
          type="button"
          aria-label="Clear filter"
          onClick={() => onChange("")}
          className="grid size-[18px] shrink-0 place-items-center rounded-full text-dim hover:bg-hover"
        >
          <Icons.close size={12} />
        </button>
      )}
    </div>
  );
};

export default FilterBox;
