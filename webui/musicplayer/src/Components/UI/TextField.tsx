import { forwardRef, useId } from "react";
import type { InputHTMLAttributes, TextareaHTMLAttributes } from "react";
import cn from "./cn";

type CommonProps = {
  label?: string;
  /** Validation message from react-hook-form; turns the field red. */
  error?: string;
  /** Muted line under the field explaining what it accepts. */
  hint?: string;
  className?: string;
  /** Use the mono face — filters and urls read better in it. */
  mono?: boolean;
};

export type TextFieldProps = CommonProps &
  Omit<InputHTMLAttributes<HTMLInputElement>, "className">;

const fieldShell = (error?: string) =>
  cn(
    "w-full rounded-control border bg-raised px-[10px] text-xs text-fg",
    "placeholder:text-muted transition-colors",
    error ? "border-syntax-error" : "border-line focus:border-accent"
  );

/**
 * The desktop's `FormField`: a tracked-out caption over a 32px input whose
 * border picks up the accent while focused.
 *
 * `forwardRef` because every form in the app is a react-hook-form, and
 * `register()` hands back a ref this has to pass through to the input.
 */
export const TextField = forwardRef<HTMLInputElement, TextFieldProps>(
  ({ label, error, hint, className, mono, id, ...props }, ref) => {
    const generated = useId();
    const fieldId = id ?? generated;
    return (
      <div className={cn("flex flex-col gap-[5px]", className)}>
        {label && (
          <label
            htmlFor={fieldId}
            className="text-[10px] tracking-[1px] text-muted"
          >
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={fieldId}
          aria-invalid={!!error}
          aria-describedby={error ? `${fieldId}-error` : undefined}
          className={cn("h-8", fieldShell(error), mono && "font-mono")}
          {...props}
        />
        {error ? (
          <p id={`${fieldId}-error`} className="text-[11px] text-syntax-error">
            {error}
          </p>
        ) : hint ? (
          <p className="text-[11px] text-muted">{hint}</p>
        ) : null}
      </div>
    );
  }
);

TextField.displayName = "TextField";

export type TextAreaFieldProps = CommonProps &
  Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, "className">;

export const TextAreaField = forwardRef<
  HTMLTextAreaElement,
  TextAreaFieldProps
>(({ label, error, hint, className, mono, id, rows = 3, ...props }, ref) => {
  const generated = useId();
  const fieldId = id ?? generated;
  return (
    <div className={cn("flex flex-col gap-[5px]", className)}>
      {label && (
        <label
          htmlFor={fieldId}
          className="text-[10px] tracking-[1px] text-muted"
        >
          {label}
        </label>
      )}
      <textarea
        ref={ref}
        id={fieldId}
        rows={rows}
        aria-invalid={!!error}
        aria-describedby={error ? `${fieldId}-error` : undefined}
        className={cn("resize-none py-2", fieldShell(error), mono && "font-mono")}
        {...props}
      />
      {error ? (
        <p id={`${fieldId}-error`} className="text-[11px] text-syntax-error">
          {error}
        </p>
      ) : hint ? (
        <p className="text-[11px] text-muted">{hint}</p>
      ) : null}
    </div>
  );
});

TextAreaField.displayName = "TextAreaField";

export default TextField;
