import React, {FC} from 'react';
import Albums from './Albums';
import {albums} from '../../Mocks/Albums';

const AlbumsWithData: FC = () => {
  return <Albums albums={albums} onAlbumPress={() => {}} onSeeAll={() => {}} />;
};

export default AlbumsWithData;
