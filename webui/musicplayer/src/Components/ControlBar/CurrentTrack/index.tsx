import { FC } from "react";
import { usePlayback } from "../../../Hooks/usePlayback";
import { usePlayackProgress } from "../../../Hooks/usePlaybackProgress";
import CurrentTrack from "./CurrentTrack";

const CurrentTrackWithData: FC = () => {
  const { nowPlaying } = usePlayback();
  const { progress } = usePlayackProgress();
  return <CurrentTrack nowPlaying={{ ...nowPlaying, progress: progress! }} />;
};

export default CurrentTrackWithData;
