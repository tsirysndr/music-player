import { useEffect, useRef } from "react";

export type LoadMoreProps = {
  hasMore?: boolean;
  onLoadMore?: () => void;
};

/**
 * A sentinel that pages the next batch in as it scrolls into view.
 *
 * `rootMargin` fires it a screen early so the list grows before the user
 * reaches the bottom, which is what makes it read as one long list rather
 * than as pages.
 */
const LoadMore = ({ hasMore, onLoadMore }: LoadMoreProps) => {
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
  return <div ref={ref} className="h-10 w-full" aria-hidden="true" />;
};

export default LoadMore;
