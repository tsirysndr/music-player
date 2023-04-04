import React, {FC} from 'react';
import Songs from './Songs';
import {tracks} from '../../Mocks/Tracks';

const SongsWithData: FC = () => {
  return <Songs tracks={tracks} onPressTrack={() => {}} onSeeAll={() => {}} />;
};

export default SongsWithData;
