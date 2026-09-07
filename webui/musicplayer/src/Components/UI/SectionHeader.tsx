import cn from "./cn";

export type SectionHeaderProps = {
  title: string;
  className?: string;
};

/** The desktop's `SectionHeader`: a tracked-out caption with a rule after it. */
const SectionHeader = ({ title, className }: SectionHeaderProps) => (
  <div className={cn("flex h-[30px] items-center gap-[10px]", className)}>
    <span className="text-[10px] tracking-[1.5px] text-muted">{title}</span>
    <span className="h-px flex-1 bg-line" />
  </div>
);

export default SectionHeader;
