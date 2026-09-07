import type { ReactNode } from "react";
import cn from "./cn";
import type { IconComponent } from "./icons";

export type EmptyStateProps = {
  icon?: IconComponent;
  title: string;
  /** One line of what to do about it, as the desktop's empty views give. */
  hint?: string;
  action?: ReactNode;
  className?: string;
};

/** The centred "nothing here yet" the desktop shows in place of a list. */
const EmptyState = ({
  icon: Icon,
  title,
  hint,
  action,
  className,
}: EmptyStateProps) => (
  <div
    className={cn(
      "flex flex-1 flex-col items-center justify-center gap-3 px-6 py-16 text-center",
      className
    )}
  >
    {Icon && <Icon size={34} className="text-muted opacity-60" />}
    <p className="text-[13px] text-muted">{title}</p>
    {hint && <p className="max-w-md text-xs text-muted opacity-80">{hint}</p>}
    {action}
  </div>
);

export default EmptyState;
