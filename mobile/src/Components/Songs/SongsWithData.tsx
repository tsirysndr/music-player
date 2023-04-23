import React, {FC} from 'react';
import Songs from './Songs';
import {useRecoilState} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';
import {useGetTracksQuery} from '../../Hooks/GraphQL';
import {useNavigation} from '@react-navigation/native';
import {playQueueState} from '../PlayQueue/PlayQueueState';

const SongsWithData: FC = () => {
  const navigation = useNavigation<any>();
  const {data, loading} = useGetTracksQuery({
    variables: {
      limit: 10,
    },
  });
  const tracks = !loading && data ? data.tracks : [];
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  const [playQueue, setPlayQueue] = useRecoilState(playQueueState);
  const onSeeAll = () => navigation.navigate('Tracks');
  const onPressTrack = (track: any) => {
    setCurrentTrack(track);
    setPlayQueue({
      ...playQueue,
      previousTracks: playQueue.previousTracks.concat(track),
      position: playQueue.previousTracks.length,
    });
  };
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
      currentTrack={currentTrack}
      onPressTrack={onPressTrack}
      onSeeAll={onSeeAll}
    />
  );
};

export default SongsWithData;
