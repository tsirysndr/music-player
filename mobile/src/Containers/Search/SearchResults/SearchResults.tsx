import React, {FC} from 'react';
import {FlatList, Text} from 'react-native';
import TrackRow from '../../../Components/TrackRow';
import AlbumRow from '../../../Components/AlbumRow';
import ArtistRow from '../../../Components/ArtistRow';

export type SearchResultsProps = {
  currentFilter: string;
  query: string;
  results: {
    tracks: any[];
    albums: any[];
    artists: any[];
    playlists: any[];
  };
  onPressAlbum: (album: any) => void;
  onPressArtist: (artist: any) => void;
};

const SearchResults: FC<SearchResultsProps> = ({
  currentFilter,
  results,
  query,
  onPressArtist,
}) => {
  return (
    <>
      {currentFilter === 'Tracks' && query !== '' && (
        <FlatList
          data={results.tracks}
          renderItem={({item}) => <TrackRow track={item} />}
        />
      )}
      {currentFilter === 'Albums' && query !== '' && (
        <FlatList
          data={results.albums}
          renderItem={({item}) => <AlbumRow album={item} />}
        />
      )}
      {currentFilter === 'Artists' && query !== '' && (
        <FlatList
          data={results.artists}
          renderItem={({item}) => (
            <ArtistRow artist={item} onSelected={() => onPressArtist(item)} />
          )}
        />
      )}
      {currentFilter === 'Playlists' && query !== '' && (
        <FlatList
          data={results.playlists}
          renderItem={({item}) => <Text>{item.name}</Text>}
        />
      )}
    </>
  );
};

export default SearchResults;
