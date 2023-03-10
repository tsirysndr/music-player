import { FC } from "react";
import ControlBar from "./ControlBar";
import { useDevices } from "../../Hooks/useDevices";
import { usePlayback } from "../../Hooks/usePlayback";

const ControlBarWithData: FC = () => {
  const {
    castDevices,
    currentCastDevice,
    connectToCastDevice,
    disconnectFromCastDevice,
  } = useDevices();

  const {
    play,
    pause,
    next,
    previous,
    nowPlaying,
    nextTracks,
    previousTracks,
    playTrackAt,
    removeTrackAt,
  } = usePlayback();
  return (
    <ControlBar
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      nowPlaying={nowPlaying}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      castDevices={castDevices}
      currentCastDevice={currentCastDevice}
      connectToCastDevice={(id) => connectToCastDevice({ variables: { id } })}
      disconnectFromCastDevice={() => disconnectFromCastDevice()}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
    />
  );
};

export default ControlBarWithData;
