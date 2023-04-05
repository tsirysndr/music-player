import React from 'react';
import CurrentTrack from './CurrentTrack';
import {useRecoilState} from 'recoil';
import {currentTrackState} from './CurrentTrackState';
import {playQueueState} from '../PlayQueue/PlayQueueState';

const CurrentTrackWithData = () => {
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  const [{previousTracks, nextTracks, position}, setPlayQueue] =
    useRecoilState(playQueueState);
  const tracks = [...previousTracks, ...nextTracks];

  const onPageSelected = (e: any) => {
    setPlayQueue({
      nextTracks: tracks.slice(e.nativeEvent.position),
      previousTracks: tracks.slice(0, e.nativeEvent.position),
      position: e.nativeEvent.position,
    });
    setCurrentTrack(tracks[e.nativeEvent.position]);
  };

  const onLike = () => {
    setCurrentTrack({
      ...currentTrack!,
      liked: !currentTrack!.liked,
    });
  };

  return (
    <CurrentTrack
      track={currentTrack}
      initialPage={position}
      onPageSelected={onPageSelected}
      tracks={tracks}
      onLike={onLike}
    />
  );
};

export default CurrentTrackWithData;
