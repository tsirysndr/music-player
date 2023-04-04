import React, {FC} from 'react';
import Artists from './Artists';
import {artists} from '../../Mocks/Artists';

const ArtistsWithData: FC = () => {
  return (
    <Artists artists={artists} onSeeAll={() => {}} onArtistPress={() => {}} />
  );
};

export default ArtistsWithData;
