import React, {FC, useEffect} from 'react';
import styled from '@emotion/native';
import Feather from 'react-native-vector-icons/Feather';
import FontAwesome from 'react-native-vector-icons/FontAwesome5';
import Ionicons from 'react-native-vector-icons/Ionicons';
import {Dimensions} from 'react-native';
import DefaultPagerView from 'react-native-pager-view';
import {Track} from '../../Types';
import {useCover} from '../../Hooks/useCover';

const NoAlbumCover = styled.View<{width: number}>`
  width: 180px;
  height: 180px;
  background-color: #161515;
  margin-right: 8px;
  margin-left: 8px;
  border-radius: 10px;
  align-items: center;
  justify-content: center;
  ${({width}) => `width: ${width}px`};
  ${({width}) => `height: ${width}px`};
`;

const Cover = styled.Image<{width: number}>`
  margin-right: 8px;
  margin-left: 8px;
  border-radius: 10px;
  ${({width}) => `width: ${width}px`};
  ${({width}) => `height: ${width}px`};
`;

const Container = styled.View`
  width: 100%;
`;

const TrackRow = styled.View`
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  height: 120px;
  margin-left: 30px;
  margin-right: 30px;
`;

const TrackInfo = styled.View`
  flex-direction: column;
  flex: 1;
`;

const TrackTitle = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 20px;
`;

const TrackArtist = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 3px;
`;

const Button = styled.TouchableOpacity`
  height: 40px;
  width: 40px;
  justify-content: center;
  align-items: flex-end;
`;

const PagerView = styled(DefaultPagerView)<{height: number}>`
  width: 100%;
  align-items: center;
  justify-content: center;
  ${({height}) => `height: ${height}px`};
`;

const CoverWrapper = styled.View`
  align-items: center;
  justify-content: center;
`;

export type AlbumCoverProps = {
  cover?: string;
};

const AlbumCover: FC<AlbumCoverProps> = props => {
  const screenWidth = Math.round(Dimensions.get('window').width);
  const cover = useCover(props.cover);
  return (
    <CoverWrapper>
      {props.cover && <Cover source={{uri: cover}} width={screenWidth - 60} />}
      {!props.cover && (
        <NoAlbumCover width={screenWidth - 60}>
          <Feather name="disc" size={200} color="#a7a7a93a" />
        </NoAlbumCover>
      )}
    </CoverWrapper>
  );
};

export type CurrentTrackProps = {
  track?: Track;
  onPageSelected: (event: any) => void;
  initialPage: number;
  tracks: any;
  onLike: () => void;
};

const CurrentTrack: FC<CurrentTrackProps> = props => {
  const screenWidth = Math.round(Dimensions.get('window').width);
  const {initialPage, onPageSelected, track, tracks, onLike} = props;
  useEffect(() => {
    (pagerRef.current as any).setPage(initialPage);
  }, [initialPage]);
  const pagerRef = React.useRef(null);
  return (
    <Container>
      <>
        <PagerView
          ref={pagerRef}
          height={screenWidth - 60}
          initialPage={initialPage}
          onPageSelected={onPageSelected}>
          {tracks.map((item: any) => (
            <AlbumCover key={item.id} cover={item.cover} />
          ))}
        </PagerView>
        <TrackRow>
          <TrackInfo>
            <TrackTitle>{track?.title}</TrackTitle>
            <TrackArtist>{track?.artist}</TrackArtist>
          </TrackInfo>
          {!track?.liked && (
            <Button onPress={onLike}>
              <Ionicons name="heart-outline" size={24} color="#cbcbcb" />
            </Button>
          )}
          {track?.liked && (
            <Button onPress={onLike}>
              <Ionicons name="heart" size={24} color="#ab28fc" />
            </Button>
          )}
          <Button>
            <FontAwesome name="ellipsis-v" size={20} color="#cbcbcb" />
          </Button>
        </TrackRow>
      </>
    </Container>
  );
};

export default CurrentTrack;
