import { FC, useState } from "react";
import { AppShell } from "../Layout";
import {
  AlbumCard,
  ArtistRow,
  EmptyState,
  FilterBox,
  Icons,
  TrackListHeader,
  TrackRow,
  cn,
  type AlbumCardItem,
  type ArtistRowItem,
  type PlaylistOption,
  type TrackRowItem,
} from "../UI";

type Tab = "tracks" | "albums" | "artists";

export type SearchResultsProps = {
  query: string;
  tracks: TrackRowItem[];
  albums: AlbumCardItem[];
  artists: ArtistRowItem[];
  currentTrackId?: string;
  recentPlaylists: PlaylistOption[];
  onSearch: (query: string) => void;
  onPlayTrack: (id: string) => void;
  onPlayNext: (id: string) => void;
  onToggleLike: (id: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onPlayAlbum: (id: string, shuffle: boolean) => void;
  onPlayArtist: (id: string) => void;
};

/**
 * Library search. The desktop opens a command palette over the window; on the
 * web this is a page, so the query survives a reload and can be linked to.
 */
const SearchResults: FC<SearchResultsProps> = ({
  query,
  tracks,
  albums,
  artists,
  currentTrackId,
  recentPlaylists,
  onSearch,
  onPlayTrack,
  onPlayNext,
  onToggleLike,
  onAddTrackToPlaylist,
  onPlayAlbum,
  onPlayArtist,
}) => {
  const [tab, setTab] = useState<Tab>("tracks");

  const counts: Record<Tab, number> = {
    tracks: tracks.length,
    albums: albums.length,
    artists: artists.length,
  };
  const total = counts.tracks + counts.albums + counts.artists;

  return (
    <AppShell title="Search">
      {/* Not in the header: the header names where you are, and this is the
          page's own control. */}
      <FilterBox
        value={query}
        placeholder="Search your library…"
        autoFocus
        className="mb-4 w-full sm:max-w-[420px]"
        aria-label="Search your library"
        onChange={onSearch}
      />

      {query.trim() === "" ? (
        <EmptyState
          icon={Icons.search}
          title="Search your library"
          hint="Tracks, albums and artists. Press / from anywhere to get back here."
        />
      ) : total === 0 ? (
        <EmptyState icon={Icons.search} title={`Nothing matches “${query}”`} />
      ) : (
        <>
          <div className="mb-4 flex gap-2 border-b border-line">
            {(["tracks", "albums", "artists"] as const).map((value) => (
              <button
                key={value}
                type="button"
                onClick={() => setTab(value)}
                className={cn(
                  "-mb-px border-b-2 px-3 pb-2 text-xs font-semibold capitalize transition-colors",
                  tab === value
                    ? "border-accent text-accent"
                    : "border-transparent text-dim hover:text-fg"
                )}
              >
                {value}
                <span className="ml-[6px] font-mono text-[10px] text-muted">
                  {counts[value]}
                </span>
              </button>
            ))}
          </div>

          {tab === "tracks" &&
            (tracks.length === 0 ? (
              <EmptyState icon={Icons.music} title="No matching tracks" />
            ) : (
              <>
                <TrackListHeader />
                <div className="flex flex-col">
                  {tracks.map((track, index) => (
                    <TrackRow
                      key={track.id}
                      track={track}
                      index={index}
                      current={track.id === currentTrackId}
                      playlists={recentPlaylists}
                      onPlay={() => onPlayTrack(track.id)}
                      onLike={() => onToggleLike(track.id)}
                      onPlayNext={() => onPlayNext(track.id)}
                      onAddToPlaylist={(playlistId) =>
                        onAddTrackToPlaylist(playlistId, track.id)
                      }
                    />
                  ))}
                </div>
              </>
            ))}

          {tab === "albums" &&
            (albums.length === 0 ? (
              <EmptyState icon={Icons.disc} title="No matching albums" />
            ) : (
              <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
                {albums.map((album) => (
                  <AlbumCard
                    key={album.id}
                    album={album}
                    onPlay={() => onPlayAlbum(album.id, false)}
                    onShufflePlay={() => onPlayAlbum(album.id, true)}
                  />
                ))}
              </div>
            ))}

          {tab === "artists" &&
            (artists.length === 0 ? (
              <EmptyState icon={Icons.artist} title="No matching artists" />
            ) : (
              <div className="flex flex-col">
                {artists.map((artist) => (
                  <ArtistRow
                    key={artist.id}
                    artist={artist}
                    onPlay={() => onPlayArtist(artist.id)}
                  />
                ))}
              </div>
            ))}
        </>
      )}
    </AppShell>
  );
};

export default SearchResults;
