import React, {FC} from 'react';
import Songs from './Songs';
import {tracks} from '../../Mocks/Tracks';
import {useRecoilState} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';

const SongsWithData: FC = () => {
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  return (
    <Songs
      tracks={tracks}
      currentTrack={currentTrack}
      onPressTrack={setCurrentTrack}
      onSeeAll={() => {}}
    />
  );
};

export default SongsWithData;
