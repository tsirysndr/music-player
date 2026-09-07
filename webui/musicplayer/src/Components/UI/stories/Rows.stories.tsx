import type { Meta, StoryObj } from "@storybook/react-vite";
import {
  albums,
  artists,
  noop,
  playlists,
  recentPlaylists,
  tracks,
} from "../../../Stories/fixtures";
import {
  AlbumCard,
  ArtistRow,
  Icons,
  PlaylistRow,
  RadioCategoryTile,
  RadioRow,
  TrackListHeader,
  TrackRow,
  type StationItem,
} from "..";

/**
 * The list rows the library pages are built from: one row per kind, each with
 * its own hover affordances and "…" menu.
 */
const meta: Meta = {
  title: "UI/Rows",
  decorators: [
    (Story) => (
      <div className="w-full bg-window p-6">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj;

export const Tracks: Story = {
  render: () => (
    <>
      <TrackListHeader />
      {tracks.map((track, index) => (
        <TrackRow
          key={track.id}
          track={track}
          index={index}
          current={index === 1}
          playlists={recentPlaylists}
          onPlay={noop}
          onLike={noop}
          onPlayNext={noop}
          onAddToQueue={noop}
          onAddToPlaylist={noop}
        />
      ))}
    </>
  ),
};

/** On an album page the position column shows the track number instead. */
export const TracksNumbered: Story = {
  render: () => (
    <>
      <TrackListHeader />
      {tracks.map((track, index) => (
        <TrackRow
          key={track.id}
          track={track}
          index={index}
          useTrackNumber
          playlists={recentPlaylists}
          onPlay={noop}
        />
      ))}
    </>
  ),
};

/** Inside a playlist the menu grows a "Remove from playlist" entry. */
export const TracksInAPlaylist: Story = {
  render: () => (
    <>
      <TrackListHeader />
      {tracks.slice(0, 2).map((track, index) => (
        <TrackRow
          key={track.id}
          track={track}
          index={index}
          playlists={recentPlaylists}
          onPlay={noop}
          onRemove={noop}
        />
      ))}
    </>
  ),
};

export const Artists: Story = {
  render: () => (
    <div className="flex flex-col">
      {artists.map((artist) => (
        <ArtistRow key={artist.id} artist={artist} onPlay={noop} />
      ))}
    </div>
  ),
};

export const Playlists: Story = {
  render: () => (
    <div className="flex flex-col">
      {playlists.map((playlist) => (
        <PlaylistRow
          key={playlist.id}
          playlist={playlist}
          onPlay={noop}
          onEdit={noop}
          onDelete={noop}
        />
      ))}
    </div>
  ),
};

export const AlbumCards: Story = {
  render: () => (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
      {albums.map((album) => (
        <AlbumCard
          key={album.id}
          album={album}
          onPlay={noop}
          onShufflePlay={noop}
          onPlayNext={noop}
          onAddToQueue={noop}
          onLike={noop}
        />
      ))}
    </div>
  ),
};

const stations: StationItem[] = [
  {
    id: "s1",
    name: "Radio Paradise",
    subtitle: "Mellow Mix · 320kbps",
    source: "radio-browser",
    logo: "https://cdn-profiles.tunein.com/s25204/images/logog.png",
  },
  {
    id: "s2",
    name: "SomaFM Groove Salad",
    subtitle: "Ambient · 128kbps",
    source: "radio-browser",
    bookmarked: true,
  },
  { id: "s3", name: "A station with no logo and no subtitle at all" },
];

export const RadioStations: Story = {
  render: () => (
    <div className="flex flex-col">
      {stations.map((station, index) => (
        <RadioRow
          key={station.id}
          station={station}
          playing={index === 0}
          onPlay={noop}
          onBookmark={noop}
        />
      ))}
    </div>
  ),
};

export const RadioCategories: Story = {
  render: () => (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
      <RadioCategoryTile
        label="Jazz"
        term="jazz"
        icon={Icons.music}
        color="#7c5cff"
        onSelect={noop}
      />
      <RadioCategoryTile
        label="Rock"
        term="rock"
        icon={Icons.broadcast}
        color="#e04b3a"
        onSelect={noop}
      />
      <RadioCategoryTile
        label="Classical"
        term="classical"
        icon={Icons.disc}
        color="#2f9e44"
        onSelect={noop}
      />
      <RadioCategoryTile
        label="Talk"
        term="talk"
        icon={Icons.listMusic}
        color="#ffb000"
        onSelect={noop}
      />
    </div>
  ),
};
