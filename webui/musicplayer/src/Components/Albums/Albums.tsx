import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC } from "react";
import { useCover } from "../../Hooks/useCover";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import AlbumIcon from "../Icons/AlbumCover";
import { Track } from "../../Types";
import { Device } from "../../Types/Device";

const Container = styled.div`
  display: flex;
  flex-direction: row;
  background-color: ${(props) => props.theme.colors.background};
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

const AlbumCover = styled.img`
  height: 220px;
  width: 220px;
  border-radius: 3px;
  cursor: pointer;
`;

const NoAlbumCover = styled.div`
  height: 220px;
  width: 220px;
  border-radius: 3px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #ddaefb14;
`;

const Wrapper = styled.div`
  margin-top: 34px;
  margin-left: 10px;
`;

const Artist = styled.div`
  color: #828282;
  margin-bottom: 56px;
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
`;

const Title = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  color: ${(props) => props.theme.colors.text};
`;

const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;
`;

export type AlbumsProps = {
  albums: any[];
  onClickAlbum: (album: any) => void;
  onClickLibraryItem: (item: string) => void;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  nextTracks: Track[];
  previousTracks: Track[];
  onPlayNext: (id: string) => void;
  onPlayTrackAt: (position: number) => void;
  onRemoveTrackAt: (position: number) => void;
  onSearch: (query: string) => void;
  folders: any[];
  playlists: any[];
  onCreateFolder: (name: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onDeleteFolder: (id: string) => void;
  onDeletePlaylist: (id: string) => void;
  onEditFolder: (id: string, name: string) => void;
  onEditPlaylist: (id: string, name: string, description?: string) => void;
  onPlayPlaylist: (
    playlistId: string,
    shuffle: boolean,
    position?: number
  ) => void;
  devices: Device[];
  castDevices: Device[];
  currentDevice?: Device;
  connectToDevice: (deviceId: string) => void;
  disconnectFromDevice: () => void;
};

export type AlbumProps = {
  onClick: (item: any) => void;
  album: any;
};

const Album: FC<AlbumProps> = ({ onClick, album }) => {
  const { cover } = useCover(album.cover);
  return (
    <>
      {cover && <AlbumCover src={cover} onClick={() => onClick(album)} />}
      {!cover && (
        <NoAlbumCover onClick={() => onClick(album)}>
          <AlbumIcon />
        </NoAlbumCover>
      )}
      <Title onClick={() => onClick(album)}>{album.title}</Title>
      <Artist>{album.artist}</Artist>
    </>
  );
};

const Albums: FC<AlbumsProps> = (props) => {
  const { albums, onClickAlbum } = props;
  return (
    <Container>
      <Sidebar active="albums" {...props} />
      <Content>
        <ControlBar {...props} />
        <Scrollable>
          <MainContent title="Albums" placeholder="Filter Albums">
            <Wrapper>
              <Grid gridColumns={[2, 3, 4, 6]} gridMargins={[8, 16, 18]}>
                {albums.map((item) => (
                  <Cell key={item.id}>
                    <Album onClick={onClickAlbum} album={item} />
                  </Cell>
                ))}
              </Grid>
            </Wrapper>
          </MainContent>
        </Scrollable>
      </Content>
    </Container>
  );
};

Albums.defaultProps = {
  albums: [],
};

export default Albums;
