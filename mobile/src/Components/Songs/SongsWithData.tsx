import React, {FC} from 'react';
import Songs from './Songs';
import {useGetTracksQuery} from '../../Hooks/GraphQL';
import {useNavigation} from '@react-navigation/native';

const SongsWithData: FC = () => {
  const navigation = useNavigation<any>();
  const {data, loading} = useGetTracksQuery({
    variables: {
      limit: 10,
    },
  });
  const tracks = !loading && data ? data.tracks : [];
  const onSeeAll = () => navigation.navigate('Tracks');
  return (
    <Songs
      tracks={tracks.map(track => ({
        id: track.id,
        title: track.title,
        artist: track.artist,
        album: track.album.title,
        duration: track.duration!,
        cover: track.album.cover || undefined,
        artistId: track.artists[0].id,
        albumId: track.album.id,
      }))}
      onSeeAll={onSeeAll}
    />
  );
};

export default SongsWithData;
