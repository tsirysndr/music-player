import cn from "./cn";
import { Icons } from "./icons";

export type SearchButtonProps = {
  className?: string;
  onClick: () => void;
};

/**
 * The header's search affordance, from the desktop client: it opens the
 * command palette rather than editing in place, and advertises the `/`
 * shortcut that does the same thing.
 */
const SearchButton = ({ className, onClick }: SearchButtonProps) => (
  <button
    type="button"
    onClick={onClick}
    className={cn(
      "flex h-8 w-[220px] items-center gap-2 rounded-full border border-line bg-panel pl-3 pr-2",
      "transition-colors hover:bg-raised",
      className
    )}
  >
    <Icons.search size={14} className="shrink-0 text-muted" />
    <span className="flex-1 text-left text-xs text-muted">Search</span>
    <kbd className="grid h-[18px] w-5 place-items-center rounded border border-line bg-hover font-mono text-[10px] text-dim">
      /
    </kbd>
  </button>
);

export default SearchButton;
