import React from 'react';
import CurrentTrack from './CurrentTrack';
import {tracks} from '../../Mocks/Tracks';

const CurrentTrackWithData = () => {
  return (
    <CurrentTrack
      track={tracks[0]}
      initialPage={0}
      onPageSelected={_e => {
        // console.log(_e.nativeEvent.position);
      }}
      tracks={tracks}
    />
  );
};

export default CurrentTrackWithData;
