import { forwardRef, useId, type SelectHTMLAttributes } from "react";
import cn from "./cn";
import { Icons } from "./icons";

export type SelectOption = { value: string; label: string };

export type SelectProps = Omit<
  SelectHTMLAttributes<HTMLSelectElement>,
  "className" | "children"
> & {
  label?: string;
  options: SelectOption[];
  error?: string;
  className?: string;
};

/**
 * The desktop's `Dropdown`, as a native `<select>`.
 *
 * Native rather than a custom listbox on purpose: it is what gives a phone the
 * platform's own wheel picker, and there is nothing in the desktop's dropdown
 * — a flat list of short labels — that a native control cannot do. Only the
 * chrome is replaced, so it still reads as part of the skin.
 */
const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, options, error, className, id, ...props }, ref) => {
    const generated = useId();
    const fieldId = id ?? generated;
    return (
      <div className={cn("flex min-w-0 flex-col gap-[5px]", className)}>
        {label && (
          <label
            htmlFor={fieldId}
            className="text-[10px] tracking-[1px] text-muted"
          >
            {label}
          </label>
        )}
        <div className="relative">
          <select
            ref={ref}
            id={fieldId}
            aria-invalid={!!error}
            className={cn(
              "h-8 w-full appearance-none rounded-control border bg-raised pl-[10px] pr-7 text-xs text-fg",
              error ? "border-syntax-error" : "border-line focus:border-accent"
            )}
            {...props}
          >
            {options.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          <Icons.chevronLeft
            size={11}
            aria-hidden="true"
            className="pointer-events-none absolute right-[10px] top-1/2 -translate-y-1/2 -rotate-90 text-muted"
          />
        </div>
        {error && <p className="text-[11px] text-syntax-error">{error}</p>}
      </div>
    );
  }
);

Select.displayName = "Select";

export default Select;
