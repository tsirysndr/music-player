import { FC } from "react";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlayackProgress } from "../../Hooks/usePlaybackProgress";
import ControlBar from "./ControlBar";

const ControlBarWithData: FC = () => {
  const {
    play,
    pause,
    next,
    previous,
    playTrackAt,
    removeTrackAt,
    nowPlaying,
    previousTracks,
    nextTracks,
  } = usePlayback();
  const { progress } = usePlayackProgress();
  return (
    <ControlBar
      nowPlaying={{ ...nowPlaying, progress: progress! }}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
      previousTracks={previousTracks}
      nextTracks={nextTracks}
    />
  );
};

export default ControlBarWithData;
