import styled from "@emotion/styled";
import { FC } from "react";
import TracksTable from "../../TracksTable";

export type TracksProps = {
  tracks: any[];
  nowPlaying: any;
  onPlayTrack: (id: string, postion?: number) => void;
  onPlayNext: (id: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  recentPlaylists: any[];
};

const Tracks: FC<TracksProps> = (props) => {
  const {
    tracks,
    nowPlaying,
    onPlayTrack,
    onPlayNext,
    onCreatePlaylist,
    onAddTrackToPlaylist,
    recentPlaylists,
  } = props;
  return (
    <TracksTable
      tracks={tracks}
      currentTrackId={nowPlaying.id}
      isPlaying={nowPlaying.isPlaying}
      onPlayTrack={onPlayTrack}
      onPlayNext={onPlayNext}
      onCreatePlaylist={onCreatePlaylist}
      maxHeight="initial"
      recentPlaylists={recentPlaylists}
      onAddTrackToPlaylist={onAddTrackToPlaylist}
    />
  );
};

Tracks.defaultProps = {
  tracks: [],
};

export default Tracks;
