import cn from "./cn";

export type SkeletonBoxProps = {
  className?: string;
  style?: React.CSSProperties;
};

/**
 * One shimmering placeholder block, matching the desktop's `SkeletonBox`: a
 * highlight sweeping left to right over the skin's two skeleton colours.
 *
 * The sweep is a background-position animation rather than a moving child, so
 * a list of these costs one composited layer each and nothing lays out.
 */
export const SkeletonBox = ({ className, style }: SkeletonBoxProps) => (
  <div
    style={{
      backgroundImage:
        "linear-gradient(90deg, var(--skeleton-bg) 0%, var(--skeleton-fg) 50%, var(--skeleton-bg) 100%)",
      backgroundSize: "200% 100%",
      animation: "skeleton-sweep 1.4s linear infinite",
      ...style,
    }}
    className={cn("rounded bg-skeleton", className)}
  />
);

/**
 * A station placeholder that mirrors `RadioRow`'s layout, so nothing jumps
 * when the results land.
 */
export const RadioSkeletonRow = () => (
  <div className="flex h-[58px] items-center gap-3 px-3">
    <SkeletonBox className="size-10 rounded-md" />
    <div className="flex flex-1 flex-col gap-2">
      <SkeletonBox className="h-3 w-[42%]" />
      <SkeletonBox className="h-[9px] w-[26%] rounded-[3px]" />
    </div>
  </div>
);

export const RadioSkeletonList = ({ rows = 7 }: { rows?: number }) => (
  <div className="flex flex-col gap-[2px]">
    {Array.from({ length: rows }, (_, i) => (
      <RadioSkeletonRow key={i} />
    ))}
  </div>
);

/** Placeholder grid for the albums page. */
export const AlbumSkeletonGrid = ({ count = 12 }: { count?: number }) => (
  <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
    {Array.from({ length: count }, (_, i) => (
      <div key={i} className="rounded-skin bg-panel p-3">
        <SkeletonBox className="aspect-square w-full" />
        <SkeletonBox className="mt-2 h-3 w-3/4" />
        <SkeletonBox className="mt-2 h-[9px] w-1/2 rounded-[3px]" />
      </div>
    ))}
  </div>
);

/** Placeholder rows for any of the track tables. */
export const TrackSkeletonList = ({ rows = 12 }: { rows?: number }) => (
  <div className="flex flex-col">
    {Array.from({ length: rows }, (_, i) => (
      <div key={i} className="flex h-[42px] items-center gap-3 px-3">
        <SkeletonBox className="h-3 w-6" />
        <SkeletonBox className="h-3 flex-[0.45]" />
        <SkeletonBox className="h-3 flex-[0.28]" />
        <SkeletonBox className="h-3 flex-[0.27]" />
        <SkeletonBox className="h-3 w-10" />
      </div>
    ))}
  </div>
);

export default SkeletonBox;
