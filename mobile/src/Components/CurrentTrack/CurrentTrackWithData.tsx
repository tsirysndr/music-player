import React from 'react';
import CurrentTrack from './CurrentTrack';
import {tracks} from '../../Mocks/Tracks';

const CurrentTrackWithData = () => {
  return <CurrentTrack track={tracks[0]} />;
};

export default CurrentTrackWithData;
