import React, {FC} from 'react';
import Tracks from './Tracks';
import {useNavigation} from '@react-navigation/native';
import {useGetTracksQuery} from '../../Hooks/GraphQL';

const TracksWithData: FC = () => {
  const navigation = useNavigation<any>();
  const {data, loading, fetchMore} = useGetTracksQuery({
    variables: {
      offset: 0,
      limit: 10,
    },
  });
  const tracks = !loading && data ? data.tracks : [];

  const handleFetchMore = () => {
    fetchMore({
      variables: {
        offset: tracks.length,
        limit: 10,
      },
      updateQuery: (prev, {fetchMoreResult}) => {
        if (!fetchMoreResult) {
          return prev;
        }
        return {
          tracks: [...prev.tracks, ...fetchMoreResult.tracks],
        };
      },
    });
  };

  const onGoBack = () => navigation.goBack();
  return (
    <Tracks
      onGoBack={onGoBack}
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
      fetchMore={handleFetchMore}
    />
  );
};

export default TracksWithData;
