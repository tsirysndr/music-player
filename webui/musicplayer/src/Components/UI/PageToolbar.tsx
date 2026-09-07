import type { ReactNode } from "react";
import cn from "./cn";
import FilterBox from "./FilterBox";

export type PageToolbarProps = {
  /** Tabs, chips or buttons that belong to the page, at the left. */
  children?: ReactNode;
  /** Omitted for a page with nothing to filter. */
  filter?: string;
  filterPlaceholder?: string;
  onFilter?: (filter: string) => void;
  /** Anything to sit right of the search box — a rescan button, say. */
  trailing?: ReactNode;
  className?: string;
};

/**
 * The bar every list page puts above its content: whatever the page offers on
 * the left, and the search box pushed to the far right.
 *
 * It lives in the page rather than in the header on purpose. The header names
 * where you are and carries the window chrome; a filter belongs next to the
 * thing it filters, so it stays put when a page scrolls and does not read as
 * global search.
 */
const PageToolbar = ({
  children,
  filter,
  filterPlaceholder = "Filter…",
  onFilter,
  trailing,
  className,
}: PageToolbarProps) => (
  <div className={cn("mb-4 flex flex-wrap items-center gap-2", className)}>
    {children}
    {(onFilter || trailing) && (
      // `flex-1` on a phone so the box fills the row under whatever is above
      // it; fixed width once there is room beside it.
      <div className="ml-auto flex flex-1 items-center gap-2 sm:flex-none">
        {onFilter && (
          <FilterBox
            value={filter ?? ""}
            placeholder={filterPlaceholder}
            className="w-full sm:w-[220px]"
            onChange={onFilter}
          />
        )}
        {trailing}
      </div>
    )}
  </div>
);

export default PageToolbar;
