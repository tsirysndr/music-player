import styled from "@emotion/styled";
import { FC, useState } from "react";
import { Track } from "../../Types";
import ControlBar from "../ControlBar";
import Sidebar from "../Sidebar";
import { Tabs, Tab } from "baseui/tabs-motion";
import Tracks from "./Tracks";
import Albums from "./Albums";
import Artists from "./Artists";
import Playlists from "./Playlists";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

const Results = styled.div`
  overflow-y: auto;
  height: calc(100vh - 146px);
`;

export type SearchResultsProps = {
  tracks: any[];
  albums: any[];
  artists: any[];
  onClickAlbum: (album: any) => void;
  onClickArtist: (artist: any) => void;
  onClickLibraryItem: (item: string) => void;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  onPlayTrack: (id: string, postion?: number) => void;
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

const SearchResults: FC<SearchResultsProps> = (props) => {
  const [activeKey, setActiveKey] = useState<React.Key>(0);
  const { tracks, nowPlaying, onPlayTrack, onPlayNext } = props;
  return (
    <Container>
      <Sidebar active={""} {...props} />
      <Content>
        <ControlBar {...props} />
        <div>
          <Tabs
            activeKey={activeKey}
            onChange={({ activeKey }) => setActiveKey(activeKey)}
            overrides={{
              TabList: {
                style: {
                  marginLeft: "26px",
                  marginRight: "26px",
                },
              },
              TabBorder: {
                style: {
                  marginLeft: "26px",
                  marginRight: "26px",
                },
              },
            }}
          >
            <Tab title="Tracks">
              <Results>
                <Tracks {...props} />
              </Results>
            </Tab>
            <Tab title="Albums">
              <Results>
                <Albums {...props} />
              </Results>
            </Tab>
            <Tab title="Artists">
              <Results>
                <Artists {...props} />
              </Results>
            </Tab>
            <Tab title="Playlists">
              <Playlists />
            </Tab>
          </Tabs>
        </div>
      </Content>
    </Container>
  );
};

export default SearchResults;
