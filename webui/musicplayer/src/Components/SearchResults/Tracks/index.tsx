import styled from "@emotion/styled";
import { FC } from "react";
import TracksTable from "../../TracksTable";

export type TracksProps = {
  tracks: any[];
  nowPlaying: any;
  onPlayTrack: (id: string, postion?: number) => void;
  onPlayNext: (id: string) => void;
};

const Tracks: FC<TracksProps> = (props) => {
  const { tracks, nowPlaying, onPlayTrack, onPlayNext } = props;
  return (
    <TracksTable
      tracks={tracks}
      currentTrackId={nowPlaying.id}
      isPlaying={nowPlaying.isPlaying}
      onPlayTrack={onPlayTrack}
      onPlayNext={onPlayNext}
      maxHeight="initial"
    />
  );
};

Tracks.defaultProps = {
  tracks: [],
};

export default Tracks;
