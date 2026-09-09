import { FC } from "react";
import { AppShell } from "../Layout";
import {
  ArtistRow,
  EmptyState,
  Icons,
  LoadMore,
  PageToolbar,
  SkeletonBox,
  type ArtistRowItem,
} from "../UI";

export type ArtistsProps = {
  artists: ArtistRowItem[];
  loading?: boolean;
  filter: string;
  onFilter: (filter: string) => void;
  onPlayArtist: (id: string) => void;
  onLoadMore?: () => void;
  hasMore?: boolean;
  /** A further page is in flight. */
  loadingMore?: boolean;
};

/** The desktop's artists list. */
const Artists: FC<ArtistsProps> = ({
  artists,
  loading,
  filter,
  onFilter,
  onPlayArtist,
  onLoadMore,
  hasMore,
  loadingMore,
}) => (
  <AppShell>
    <PageToolbar
      filter={filter}
      filterPlaceholder="Filter artists…"
      onFilter={onFilter}
    />

    {loading && artists.length === 0 ? (
      <div className="flex flex-col gap-1">
        {Array.from({ length: 10 }, (_, i) => (
          <div key={i} className="flex h-14 items-center gap-[14px] px-3">
            <SkeletonBox className="size-10 rounded-full" />
            <SkeletonBox className="h-3 w-1/3" />
          </div>
        ))}
      </div>
    ) : artists.length === 0 ? (
      <EmptyState
        icon={Icons.artist}
        title={filter ? `Nothing matches “${filter}”` : "No artists yet"}
      />
    ) : (
      <>
        <div className="flex flex-col">
          {artists.map((artist) => (
            <ArtistRow
              key={artist.id}
              artist={artist}
              onPlay={() => onPlayArtist(artist.id)}
            />
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

export default Artists;
