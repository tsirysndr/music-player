import React, {FC, useEffect} from 'react';
import styled from '@emotion/native';
import Feather from 'react-native-vector-icons/Feather';
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
  margin-right: 15px;
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

const Button = styled.TouchableOpacity``;

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

const IconWrapper = styled.View`
  height: 40px;
  width: 40px;
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
  onContextMenu: (item: Track) => void;
};

const CurrentTrack: FC<CurrentTrackProps> = props => {
  const screenWidth = Math.round(Dimensions.get('window').width);
  const {initialPage, onPageSelected, track, tracks, onLike, onContextMenu} =
    props;
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
          {tracks.map((item: any, index: number) => (
            <AlbumCover key={index} cover={item.cover} />
          ))}
        </PagerView>
        <TrackRow>
          <TrackInfo>
            <TrackTitle numberOfLines={2}>{track?.title}</TrackTitle>
            <TrackArtist>{track?.artist}</TrackArtist>
          </TrackInfo>
          {!track?.liked && (
            <Button onPress={onLike}>
              <IconWrapper>
                <Ionicons name="heart-outline" size={24} color="#cbcbcb" />
              </IconWrapper>
            </Button>
          )}
          {track?.liked && (
            <Button onPress={onLike}>
              <IconWrapper>
                <Ionicons name="heart" size={24} color="#ab28fc" />
              </IconWrapper>
            </Button>
          )}
          <Button onPress={() => onContextMenu(track!)}>
            <IconWrapper>
              <Ionicons name="ellipsis-vertical" size={20} color="#cbcbcb" />
            </IconWrapper>
          </Button>
        </TrackRow>
      </>
    </Container>
  );
};

export default CurrentTrack;
