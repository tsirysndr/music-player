import styled from "@emotion/styled";
import { FC, useState } from "react";
import Add from "../Icons/Add";
import Heart from "../Icons/Heart";
import HeartOutline from "../Icons/HeartOutline";
import { EllipsisHorizontal } from "@styled-icons/ionicons-sharp";
import { StatefulPopover } from "baseui/popover";
import { NestedMenus, StatefulMenu } from "baseui/menu";
import TrackIcon from "../Icons/Track";
import { useCover } from "../../Hooks/useCover";
import NewPlaylistModal from "../Playlists/NewPlaylistModal";

const Container = styled.div`
  display: flex;
  flex-direction: row;
  height: 45px;
`;

const Separator = styled.div`
  width: 10px;
`;

const Hover = styled.button`
  color: transparent;
  background-color: transparent;
  border: none;
  &:hover,
  &:focus {
    color: #000;
  }
`;

const Icon = styled.div`
  cursor: pointer;
  display: flex;
  height: 45px;
  width: 24px;
  justify-content: center;
  align-items: center;
`;

const AlbumCover = styled.img`
  height: 43px;
  width: 43px;
`;

const AlbumCoverAlt = styled.div`
  height: 43px;
  width: 43px;
  background-color: #f7f7f8;
  display: flex;
  justify-content: center;
  align-items: center;
`;

const Track = styled.div`
  height: 54px;
  display: flex;
  flex-direction: row;
  align-items: center;
  padding-left: 5px;
  padding-right: 5px;
  border-bottom: 1px solid #e0e0e0cc;
`;

const Artist = styled.div`
  color: rgb(170, 170, 180);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
  overflow: hidden;
  max-width: 125px;
`;

const Title = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
  overflow: hidden;
  max-width: 125px;
`;

const TrackInfos = styled.div`
  margin-left: 10px;
  overflow: hidden;
`;

const ChildMenu: FC<{ onSelect: (item: { label: string }) => void }> = ({
  onSelect,
}) => {
  return (
    <StatefulMenu
      items={{
        __ungrouped: [
          {
            label: "Create new playlist",
          },
        ],
        /*
        RECENT: [
          {
            label: "New South",
          },
        ],
        */
      }}
      overrides={{
        List: {
          style: {
            boxShadow: "none",
          },
        },
      }}
      onItemSelect={({ item }) => onSelect(item)}
    />
  );
};

export type ContextMenuProps = {
  liked?: boolean;
  track: any;
  onPlayNext: (id: string) => void;
};

const ContextMenu: FC<ContextMenuProps> = ({ liked, track, onPlayNext }) => {
  const [isNewPlaylistModalOpen, setIsNewPlaylistModalOpen] = useState(false);
  const { cover } = useCover(track.cover);
  return (
    <Container>
      <Hover>
        <StatefulPopover
          placement="left"
          autoFocus={false}
          content={({ close }) => (
            <div style={{ width: 205 }}>
              <Track>
                {cover && <AlbumCover src={cover} />}
                {!cover && (
                  <AlbumCoverAlt>
                    <TrackIcon width={24} height={24} color="#a4a3a3" />
                  </AlbumCoverAlt>
                )}
                <TrackInfos>
                  <Title>{track.title}</Title>
                  <Artist>{track.artist}</Artist>
                </TrackInfos>
              </Track>
              <NestedMenus>
                <StatefulMenu
                  overrides={{
                    List: {
                      style: {
                        boxShadow: "none",
                      },
                    },
                    Option: {
                      props: {
                        getChildMenu: (item: { label: string }) => {
                          if (item.label === "Add to Playlist") {
                            return (
                              <ChildMenu
                                onSelect={(item: { label: string }) => {
                                  if (item.label === "Create new playlist") {
                                    setIsNewPlaylistModalOpen(true);
                                  }
                                  close();
                                }}
                              />
                            );
                          }
                          return null;
                        },
                      },
                    },
                  }}
                  items={[
                    {
                      id: "1",
                      label: "Play Next",
                    },
                    {
                      id: "2",
                      label: "Add to Playlist",
                    },
                  ]}
                  onItemSelect={({ item }) => {
                    if (item.label === "Add to Playlist") {
                      return;
                    }
                    if (item.label === "Play Next") {
                      onPlayNext(track.id);
                    }
                    close();
                  }}
                />
              </NestedMenus>
            </div>
          )}
          overrides={{
            Inner: {
              style: {
                backgroundColor: "#fff",
              },
            },
          }}
        >
          <Icon>
            <EllipsisHorizontal />
          </Icon>
        </StatefulPopover>
      </Hover>
      <Separator />
      <StatefulPopover
        autoFocus={false}
        placement="left"
        content={({ close }) => (
          <div style={{ width: 205 }}>
            <StatefulMenu
              overrides={{
                List: {
                  style: {
                    boxShadow: "none",
                  },
                },
              }}
              items={[
                {
                  label: <div>Create new playlist</div>,
                },
              ]}
              onItemSelect={() => {
                setIsNewPlaylistModalOpen(true);
                close();
              }}
            />
          </div>
        )}
        overrides={{
          Inner: {
            style: {
              backgroundColor: "#fff",
            },
          },
        }}
      >
        <Icon>
          <Add size={24} />
        </Icon>
      </StatefulPopover>
      <Separator />
      {liked && (
        <Icon>
          <Heart height={24} width={24} />
        </Icon>
      )}
      {!liked && (
        <Icon>
          <HeartOutline height={24} width={24} />
        </Icon>
      )}
      <NewPlaylistModal
        onClose={() => {
          setIsNewPlaylistModalOpen(false);
        }}
        isOpen={isNewPlaylistModalOpen}
      />
    </Container>
  );
};

ContextMenu.defaultProps = {
  liked: false,
};

export default ContextMenu;
