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
    seek,
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
      onSeek={(positionMs) => seek(positionMs)}
      nowPlaying={nowPlaying}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      castDevices={castDevices}
      currentCastDevice={currentCastDevice}
      connectToCastDevice={(id) => connectToCastDevice({ id })}
      disconnectFromCastDevice={() => disconnectFromCastDevice()}
      onPlayTrackAt={(position) => playTrackAt({ position })}
      onRemoveTrackAt={(position) => removeTrackAt({ position })}
    />
  );
};

export default ControlBarWithData;
