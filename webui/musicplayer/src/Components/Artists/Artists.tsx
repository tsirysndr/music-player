import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC } from "react";
import { Track } from "../../Types";
import ControlBar from "../ControlBar";
import Artist from "../Icons/Artist";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;
`;

const Wrapper = styled.div`
  margin-top: 34px;
`;

const ArtistCover = styled.img`
  height: 220px;
  width: 220px;
  border-radius: 110px;
  cursor: pointer;
`;

const NoArtistCover = styled.div`
  height: 220px;
  width: 220px;
  border-radius: 110px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #f3f3f3b9;
`;

const ArtistName = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  margin-top: 20px;
  margin-bottom: 18px;
  text-align: center;
  width: 220px;
`;

export type ArtistsProps = {
  artists: any[];
  onClickLibraryItem: (item: string) => void;
  onClickArtist: (artist: any) => void;
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
};

const Artists: FC<ArtistsProps> = (props) => {
  const { onClickArtist, artists } = props;
  return (
    <Container>
      <Sidebar active="artists" {...props} />
      <Content>
        <ControlBar {...props} />
        <Scrollable>
          <MainContent title="Artists" placeholder="Filter Artists">
            <Wrapper>
              <Grid gridColumns={[2, 3, 4]} gridMargins={[8, 16, 18]}>
                {artists.map((item) => (
                  <Cell key={item.id}>
                    {item.cover && (
                      <ArtistCover
                        src={item.cover}
                        onClick={() => onClickArtist(item)}
                      />
                    )}
                    {!item.cover && (
                      <NoArtistCover onClick={() => onClickArtist(item)}>
                        <Artist width={75} height={75} color="#a4a3a3" />
                      </NoArtistCover>
                    )}
                    <ArtistName>{item.name}</ArtistName>
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

Artists.defaultProps = {
  artists: [],
};

export default Artists;
