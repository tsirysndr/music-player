import { FC } from "react";
import { AppShell } from "../Layout";
import {
  EmptyState,
  Icons,
  LoadMore,
  PageToolbar,
  SkeletonBox,
  cn,
} from "../UI";

export type GenreItem = {
  id: string;
  name: string;
  /** Zero means the source did not say, not that the genre is empty. */
  trackCount: number;
};

export type GenresProps = {
  genres: GenreItem[];
  loading?: boolean;
  filter: string;
  onFilter: (filter: string) => void;
  onOpenGenre: (id: string) => void;
  onLoadMore?: () => void;
  hasMore?: boolean;
  loadingMore?: boolean;
};

/**
 * Browse by genre.
 *
 * A grid of tiles rather than a list: genres are few, their names are short,
 * and there is nothing per-row worth aligning into columns — a list would be
 * one word and a lot of empty space.
 */
const Genres: FC<GenresProps> = ({
  genres,
  loading,
  filter,
  onFilter,
  onOpenGenre,
  onLoadMore,
  hasMore,
  loadingMore,
}) => (
  <AppShell title="Genres">
    <PageToolbar
      filter={filter}
      filterPlaceholder="Filter genres…"
      onFilter={onFilter}
    />

    {loading && genres.length === 0 ? (
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
        {Array.from({ length: 12 }, (_, i) => (
          <SkeletonBox key={i} className="h-[76px] rounded-control" />
        ))}
      </div>
    ) : genres.length === 0 ? (
      <EmptyState
        icon={Icons.genre}
        title="No genres yet"
        hint="Genres come from your files' tags and from the artists in your library. Scan, and they appear here."
      />
    ) : (
      <>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
          {genres.map((genre) => (
            <button
              key={genre.id}
              type="button"
              onClick={() => onOpenGenre(genre.id)}
              className={cn(
                "group/tile flex h-[76px] flex-col justify-between rounded-control",
                "bg-panel p-3 text-left transition-colors hover:bg-hover"
              )}
            >
              <Icons.genre
                size={18}
                className="shrink-0 text-dim transition-colors group-hover/tile:text-accent"
              />
              <span className="min-w-0">
                <span className="block truncate text-[13px] font-semibold text-fg">
                  {genre.name}
                </span>
                {/* Omitted rather than shown as "0 tracks": a remote server
                    that reports no count has not said the genre is empty. */}
                {genre.trackCount > 0 && (
                  <span className="block truncate text-[11px] text-muted">
                    {genre.trackCount === 1
                      ? "1 track"
                      : `${genre.trackCount} tracks`}
                  </span>
                )}
              </span>
            </button>
          ))}
        </div>
        <LoadMore
          hasMore={hasMore}
          loading={loadingMore}
          onLoadMore={onLoadMore}
        />
      </>
    )}
  </AppShell>
);

export default Genres;
