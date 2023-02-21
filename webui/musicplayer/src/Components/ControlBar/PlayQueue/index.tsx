import styled from "@emotion/styled";
import { FC, useState } from "react";
import { Link } from "react-router-dom";
import { Track } from "../../../Types";
import { CloseOutline } from "@styled-icons/evaicons-outline";
import TrackIcon from "../../Icons/Track";
import { Play } from "@styled-icons/ionicons-sharp";
import { useTheme } from "@emotion/react";

const Container = styled.div`
  height: calc(100vh - 113px);
  width: 370px;
`;

const Header = styled.div`
  display: flex;
  flex-direction: row;
`;

const Title = styled.div`
  font-size: 14px;
  margin-left: 16px;
  margin-right: 16px;
  padding-top: 20px;
  margin-bottom: 20px;
  flex: 1;
`;

const Switch = styled(Title)`
  color: #ab28fc;
  flex: initial;
  cursor: pointer;
  -webkit-user-select: none;
  -ms-user-select: none;
  user-select: none;
`;

const List = styled.div`
  height: calc(100% - 59.5px);
  overflow-y: scroll;
`;

const ListItem = styled.div`
  display: flex;
  flex-direction: row;
  height: 64px;
  align-items: center;
  padding-left: 16px;
  padding-right: 16px;
  cursor: pointer;
  &:hover {
    background-color: ${({ theme }) => theme.colors.hover};
  }
`;

const TrackTitle = styled.div`
  font-size: 14px;
  font-family: RockfordSansMedium;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
`;

const Artist = styled.div`
  font-size: 14px;
  color: ${({ theme }) => theme.colors.secondaryText};
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
`;

const TrackDetails = styled.div`
  display: flex;
  min-width: 222px;
  flex-direction: column;
`;

const AlbumCover = styled.img<{ current?: boolean }>`
  height: 48px;
  width: 48px;
  border-radius: 4px;
  margin-right: 18px;
  cursor: pointer;
  ${({ current }) => `opacity: ${current ? 0.4 : 1};`}
`;

const AlbumCoverAlt = styled.div<{ current?: boolean }>`
  height: 48px;
  width: 48px;
  border-radius: 4px;
  cursor: pointer;
  background-color: ${(props) => props.theme.colors.cover};
  display: flex;
  justify-content: center;
  align-items: center;
  margin-right: 18px;
  ${({ current }) => `opacity: ${current ? 0 : 1};`}
`;

const Remove = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
`;

const Placeholder = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  padding-left: 20px;
  padding-right: 20px;
  font-size: 14px;
`;

export type PlayQueueProps = {
  previousTracks?: Track[];
  nextTracks?: Track[];
  currentTrack?: Track;
  onPlayTrackAt: (index: number) => void;
  onRemoveTrackAt: (index: number) => void;
};

const PlayQueue: FC<PlayQueueProps> = ({
  previousTracks,
  nextTracks,
  onPlayTrackAt,
  onRemoveTrackAt,
}) => {
  const theme = useTheme();
  const [active, setActive] = useState("playqueue");

  const onSwitch = () => {
    if (active === "playqueue") {
      setActive("history");
      return;
    }
    setActive("playqueue");
  };

  const _onPlayTrackAt = (index: number) => {
    if (active === "playqueue") {
      onPlayTrackAt((previousTracks?.length || 0) + index);
      return;
    }
    onPlayTrackAt(index);
  };

  const _onRemoveTrack = (index: number) => {
    if (active === "playqueue") {
      onRemoveTrackAt((previousTracks?.length || 0) + index);
      return;
    }
    onRemoveTrackAt(index);
  };

  const tracks = active === "playqueue" ? nextTracks! : previousTracks!;
  return (
    <Container>
      <Header>
        <Title>{active === "playqueue" ? "Play Queue" : "History"}</Title>
        <Switch onClick={onSwitch}>
          {active === "playqueue" ? "History" : "Play Queue"}
        </Switch>
      </Header>
      <List>
        {tracks.map((track, index) => (
          <ListItem key={track.id}>
            {track.cover && (
              <div className="album-cover-container">
                <AlbumCover src={track.cover} />
                <div
                  onClick={() => _onPlayTrackAt(index)}
                  className="floating-play"
                >
                  <Play size={16} color={track.cover ? "#fff" : "#000"} />
                </div>
              </div>
            )}
            {!track.cover && (
              <div className="album-cover-container">
                <AlbumCoverAlt>
                  <TrackIcon width={28} height={28} color="#a4a3a3" />
                </AlbumCoverAlt>
                <div
                  onClick={() => _onPlayTrackAt(index)}
                  className="floating-play"
                >
                  <Play size={16} color={track.cover ? "#fff" : "#000"} />
                </div>
              </div>
            )}
            <TrackDetails>
              <TrackTitle>{track.title}</TrackTitle>
              <Link
                to={`/artists/${track.artistId}`}
                style={{ textDecoration: "none" }}
              >
                <Artist>{track.artist}</Artist>
              </Link>
            </TrackDetails>
            <Remove onClick={() => _onRemoveTrack(index)}>
              <CloseOutline size={24} color={theme.colors.text} />
            </Remove>
          </ListItem>
        ))}
        {tracks.length === 0 && active === "playqueue" && (
          <Placeholder>
            No upcoming tracks. Add some to your play queue.
          </Placeholder>
        )}
        {tracks.length === 0 && active === "history" && (
          <Placeholder>
            No history. Play some tracks to see them here.
          </Placeholder>
        )}
      </List>
    </Container>
  );
};

PlayQueue.defaultProps = {
  previousTracks: [],
  nextTracks: [],
};

export default PlayQueue;
