import { useEffect, useRef } from "react";

export type LoadMoreProps = {
  hasMore?: boolean;
  onLoadMore?: () => void;
  /** A page is in flight, so the sentinel shows it is working. */
  loading?: boolean;
};

/**
 * A sentinel that pages the next batch in as it scrolls into view.
 *
 * `rootMargin` fires it a screen early so the list grows before the user
 * reaches the bottom, which is what makes it read as one long list rather
 * than as pages.
 */
const LoadMore = ({ hasMore, onLoadMore, loading }: LoadMoreProps) => {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const node = ref.current;
    if (!node || !hasMore || !onLoadMore) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) onLoadMore();
      },
      { rootMargin: "600px" }
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [hasMore, onLoadMore]);

  if (!hasMore) return null;
  return (
    <div
      ref={ref}
      className="flex h-12 w-full items-center justify-center"
      // Announced rather than silent: a list that grows under you is worth a
      // word to a screen reader, and the dots are meaningless to one.
      role="status"
      aria-label={loading ? "Loading more" : undefined}
    >
      {loading && (
        <span className="flex items-center gap-1" aria-hidden="true">
          {[0, 1, 2].map((index) => (
            <span
              key={index}
              className="size-[6px] animate-pulse rounded-full bg-muted"
              // Staggered, so it reads as motion rather than as one blink.
              style={{ animationDelay: `${index * 160}ms` }}
            />
          ))}
        </span>
      )}
    </div>
  );
};

export default LoadMore;
