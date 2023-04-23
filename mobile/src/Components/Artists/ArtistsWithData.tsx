import React, {FC} from 'react';
import Artists from './Artists';
import {useGetArtistsQuery} from '../../Hooks/GraphQL';

const ArtistsWithData: FC = () => {
  const {data, loading} = useGetArtistsQuery({
    variables: {
      limit: 10,
    },
  });
  const artists = !loading && data ? data.artists : [];
  return (
    <Artists
      artists={artists.map(artist => ({
        id: artist.id,
        name: artist.name,
        cover: artist.picture,
      }))}
      onSeeAll={() => {}}
      onArtistPress={() => {}}
    />
  );
};

export default ArtistsWithData;
