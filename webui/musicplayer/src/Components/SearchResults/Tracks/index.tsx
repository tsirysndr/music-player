import styled from "@emotion/styled";
import { FC } from "react";
import { usePlayback } from "../../../Hooks/usePlayback";
import TracksTable from "../../TracksTable";

export type TracksProps = {
  tracks: any[];
  onPlayTrack: (id: string, postion?: number) => void;
  onPlayNext: (id: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  recentPlaylists: any[];
};

const Tracks: FC<TracksProps> = (props) => {
  const {
    tracks,
    onPlayTrack,
    onPlayNext,
    onCreatePlaylist,
    onAddTrackToPlaylist,
    recentPlaylists,
  } = props;
  const { currentTrackId, isPlaying } = usePlayback();
  return (
    <TracksTable
      tracks={tracks}
      currentTrackId={currentTrackId}
      isPlaying={isPlaying}
      onPlayTrack={onPlayTrack}
      onPlayNext={onPlayNext}
      onCreatePlaylist={onCreatePlaylist}
      maxHeight={"calc(100vh - 98px)"}
      recentPlaylists={recentPlaylists}
      onAddTrackToPlaylist={onAddTrackToPlaylist}
    />
  );
};

Tracks.defaultProps = {
  tracks: [],
};

export default Tracks;
