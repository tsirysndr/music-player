import React, {FC} from 'react';
import Tracks from './Tracks';
import {tracks} from '../../Mocks/Tracks';

const TracksWithData: FC = () => {
  return <Tracks onGoBack={() => {}} tracks={tracks} />;
};

export default TracksWithData;
