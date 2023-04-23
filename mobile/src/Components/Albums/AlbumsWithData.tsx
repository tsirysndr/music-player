import React, {FC} from 'react';
import Albums from './Albums';
import {useGetAlbumsQuery} from '../../Hooks/GraphQL';

const AlbumsWithData: FC = () => {
  const {data, loading} = useGetAlbumsQuery({
    variables: {
      limit: 10,
    },
  });
  const albums = !loading && data ? data.albums : [];
  return (
    <Albums
      albums={albums.map(album => ({
        id: album.id,
        title: album.title,
        artist: album.artist,
        cover: album.cover,
      }))}
      onAlbumPress={() => {}}
      onSeeAll={() => {}}
    />
  );
};

export default AlbumsWithData;
