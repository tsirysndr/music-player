import { FC } from "react";
import { AppShell } from "../Layout";
import {
  AlbumCard,
  AlbumSkeletonGrid,
  EmptyState,
  Icons,
  LoadMore,
  PageToolbar,
  type AlbumCardItem,
} from "../UI";

export type AlbumsProps = {
  albums: AlbumCardItem[];
  loading?: boolean;
  filter: string;
  onFilter: (filter: string) => void;
  onPlayAlbum: (id: string) => void;
  onShuffleAlbum: (id: string) => void;
  onLoadMore?: () => void;
  hasMore?: boolean;
  /** A further page is in flight. */
  loadingMore?: boolean;
};

/**
 * The desktop's albums grid. The desktop sizes its cells at ~196px and packs
 * as many as fit; the same intent here is a responsive grid, so a phone gets
 * two columns and a wide window six.
 */
const Albums: FC<AlbumsProps> = ({
  albums,
  loading,
  filter,
  onFilter,
  onPlayAlbum,
  onShuffleAlbum,
  onLoadMore,
  hasMore,
  loadingMore,
}) => (
  <AppShell>
    <PageToolbar
      filter={filter}
      filterPlaceholder="Filter albums…"
      onFilter={onFilter}
    />

    {loading && albums.length === 0 ? (
      <AlbumSkeletonGrid />
    ) : albums.length === 0 ? (
      <EmptyState
        icon={Icons.disc}
        title={filter ? `Nothing matches “${filter}”` : "No albums yet"}
        hint={
          filter
            ? undefined
            : "Scan a folder with `music-player scan` and they will show up here."
        }
      />
    ) : (
      <>
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
          {albums.map((album) => (
            <AlbumCard
              key={album.id}
              album={album}
              onPlay={() => onPlayAlbum(album.id)}
              onShufflePlay={() => onShuffleAlbum(album.id)}
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

export default Albums;
