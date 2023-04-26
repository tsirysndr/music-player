import React, {FC, useState} from 'react';
import Filter from './Filter';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {NativeStackScreenProps} from '@react-navigation/native-stack';
import {useNavigation} from '@react-navigation/native';
import {
  useGetAlbumsLazyQuery,
  useGetArtistsLazyQuery,
  useGetTracksLazyQuery,
} from '../../Hooks/GraphQL';
import {Album, Artist, Track} from '../../Types';

type Props = NativeStackScreenProps<MainStackParamList, 'Filter'>;

const FilterWithData: FC<Props> = ({route}) => {
  const {params} = route;
  const [albums, setAlbums] = useState<Album[]>([]);
  const [artists, setArtists] = useState<Artist[]>([]);
  const [tracks, setTracks] = useState<Track[]>([]);
  const navigation = useNavigation<any>();
  const [getAlbums] = useGetAlbumsLazyQuery();
  const [getArtists] = useGetArtistsLazyQuery();
  const [getTracks] = useGetTracksLazyQuery();
  const placeholder: Record<string, string> = {
    track: 'Filter Tracks',
    album: 'Filter Albums',
    artist: 'Filter Artists',
  };
  const onSearch = (query: string) => {
    switch (params.type) {
      case 'track':
        getTracks({
          variables: {
            filter: query,
          },
        }).then(({data}) => {
          data?.tracks &&
            setTracks(
              data.tracks.map(track => ({
                id: track.id,
                title: track.title,
                artist: track.artist,
                album: track.album.title,
                duration: track.duration!,
                cover: track.album.cover || undefined,
                artistId: track.artists[0].id,
                albumId: track.album.id,
              })),
            );
        });
        break;
      case 'album':
        getAlbums({
          variables: {
            filter: query,
          },
        }).then(({data}) => {
          data?.albums &&
            setAlbums(
              data.albums.map(album => ({
                id: album.id,
                title: album.title,
                artist: album.artist,
                cover: album.cover!,
                year: album.year!,
              })),
            );
        });
        break;
      case 'artist':
        getArtists({
          variables: {
            filter: query,
          },
        }).then(({data}) => {
          data?.artists &&
            setArtists(
              data.artists.map(artist => ({
                id: artist.id,
                name: artist.name,
                cover: artist.picture,
              })),
            );
        });
        break;
      default:
        break;
    }
  };
  const onPressArtist = (artist: any) => {
    navigation.navigate('ArtistDetails', {artist});
  };
  const onPressAlbum = (album: any) => {
    navigation.navigate('AlbumDetails', {album});
  };
  return (
    <Filter
      onGoBack={() => navigation.goBack()}
      placeholder={placeholder[params.type]}
      onSearch={onSearch}
      onPressArtist={onPressArtist}
      onPressAlbum={onPressAlbum}
      albums={albums}
      tracks={tracks}
      artists={artists}
    />
  );
};

export default FilterWithData;
