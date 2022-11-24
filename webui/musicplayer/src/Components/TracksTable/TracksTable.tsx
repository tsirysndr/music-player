import styled from "@emotion/styled";
import { Play } from "@styled-icons/ionicons-sharp";

import _, { max } from "lodash";
import { FC, memo, useMemo } from "react";
import { Link } from "react-router-dom";
import ContextMenu from "../ContextMenu";
import Speaker from "../Icons/Speaker";
import TrackIcon from "../Icons/Track";
import { FixedSizeList as List } from "react-window";
import InfiniteLoader from "react-window-infinite-loader";
import AutoSizer from "react-virtualized-auto-sizer";

const LOADING = 1;
const LOADED = 2;
let itemStatusMap: any = {};

const TableWrapper = styled.div`
  margin-top: 31px;
`;

const AlbumCoverAlt = styled.div<{ current: boolean }>`
  height: 43px;
  width: 43px;
  background-color: #f7f7f8;
  display: flex;
  justify-content: center;
  align-items: center;
  margin-right: 10px;
  ${({ current }) => `opacity: ${current ? 0 : 1};`}
`;

const CellWrapper = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-end;
  padding-right: 20px;
  height: 45px;
`;

const AlbumCoverDefault = styled.img<{ current: boolean }>`
  height: 43px;
  width: 43px;
  margin-right: 10px;
  ${({ current }) => `opacity: ${current ? 0.4 : 1};`}
`;

const Table = styled.div`
  overflow-y: auto;
`;

const TableRow = styled.div`
  display: flex;
`;

const TableHead = styled.div`
  display: flex;
  flex-direction: row;
  padding: 16px;
`;

const TableCell = styled.div`
  flex: 1;
  font-size: 14px;
  padding: 16px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
`;
type AlbumCoverProps = {
  src: string;
  current: boolean;
  className?: string;
};

const AlbumCover: FC<AlbumCoverProps> = memo((props) => (
  <AlbumCoverDefault {...props} />
));

const TableHeadCell = styled.div`
  flex: 1;
`;

const convertToLink = (row: any, item: string) => {
  switch (item) {
    case "Artist":
      return (
        <Link to={`/artists/${_.get(row, "artistId", "")}`}>
          {_.get(row, _.toLower(item), "")}
        </Link>
      );
    case "Album":
      return (
        <Link to={`/albums/${_.get(row, "albumId", "")}`}>
          {_.get(row, _.toLower(item), "")}
        </Link>
      );
    default:
      return _.get(row, _.toLower(item), "");
  }
};

export type CellProps = {
  current?: string | boolean;
  row: any;
  item: any;
  index: number;
  onPlayTrack: (id: string, postion?: number) => void;
  isAlbumTracks: boolean;
};

const Cell: FC<CellProps> = ({
  current,
  row,
  item,
  onPlayTrack,
  index,
  isAlbumTracks,
}) => {
  const AlbumCoverMemoized = useMemo(() => AlbumCover, [row.cover]);
  return (
    <CellWrapper>
      {!isAlbumTracks && item === "Title" && !row.cover && (
        <AlbumCoverAlt className="album-cover" current={!!current}>
          <TrackIcon width={24} height={24} color="#a4a3a3" />
        </AlbumCoverAlt>
      )}
      {!isAlbumTracks && item === "Title" && row.cover && (
        <AlbumCoverMemoized
          className="album-cover"
          src={row.cover}
          current={!!current}
        />
      )}
      {current && isAlbumTracks && (
        <div style={{ flex: 1 }}>
          {item === "#" && (
            <>
              <div
                style={{
                  position: "absolute",
                  left: isAlbumTracks ? 10 : 20,
                  marginTop: -11,
                }}
              >
                <Speaker color="#ab28fc" />
              </div>
              <div style={{ width: 20 }}></div>
            </>
          )}

          {item !== "#" && (
            <div
              style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis" }}
            >
              {convertToLink(row, item)}
            </div>
          )}
        </div>
      )}
      {current && !isAlbumTracks && (
        <div style={{ flex: 1 }}>
          {item === "Title" && (
            <div
              style={{
                position: "absolute",
                left: isAlbumTracks ? 20 : 37,
              }}
            >
              <Speaker color={row.cover ? "#fff" : "#ab28fc"} />
            </div>
          )}
          <div
            style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis" }}
          >
            {convertToLink(row, item)}
          </div>
        </div>
      )}
      {!current && !isAlbumTracks && item === "Title" && (
        <div
          onClick={() => onPlayTrack(row.artistId, row.index)}
          className="floating-play"
        >
          <Play size={16} color={row.cover ? "#fff" : "#000"} />
        </div>
      )}
      {!current && item === "#" && (
        <>
          <div
            onClick={() => onPlayTrack(row.albumId, row.index)}
            className="play"
          >
            <Play size={16} />
          </div>
          <div className="tracknumber">{convertToLink(row, item)}</div>
        </>
      )}
      {!current && item !== "#" && (
        <div style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis" }}>
          {convertToLink(row, item)}
        </div>
      )}
    </CellWrapper>
  );
};

const Track: FC<any> = ({
  row,
  item,
  currentIndex,
  currentTrackId,
  isPlaying,
  index,
  header,
  onPlayTrack,
}) => {
  const current =
    (item === "Title" || item === "#") &&
    ((currentIndex && currentIndex === row.index) ||
      (currentTrackId && row.id === currentTrackId)) &&
    isPlaying;
  return (
    <TableCell
      style={{
        maxWidth: item === "Time" ? "100px" : "initial",
        flex:
          item === "Time" || item === "#"
            ? "initial"
            : item === "Title"
            ? 1
            : 0.6,
      }}
    >
      <Cell
        current={current}
        row={row}
        item={item}
        index={index}
        onPlayTrack={onPlayTrack}
        isAlbumTracks={header.includes("#")}
      />
    </TableCell>
  );
};

export type TracksTableProps = {
  tracks: any[];
  header?: string[];
  title?: JSX.Element;
  currentIndex?: number;
  currentTrackId?: string;
  isPlaying?: boolean;
  maxHeight?: string;
  onPlayTrack: (id: string, position?: number) => void;
  onPlayNext: (id: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  recentPlaylists: any[];
};

const TracksTable: FC<TracksTableProps> = ({
  tracks,
  header,
  title,
  currentIndex,
  currentTrackId,
  isPlaying,
  maxHeight,
  onPlayTrack,
  onPlayNext,
  onCreatePlaylist,
  onAddTrackToPlaylist,
  recentPlaylists,
}) => {
  const isItemLoaded = (index: number) => !!itemStatusMap[index];
  const loadMoreItems: (
    startIndex: number,
    stopIndex: number
  ) => void | Promise<any> = (startIndex: number, stopIndex: number) => {
    console.log(">> loadMoreItems", startIndex, stopIndex);
    for (let index = startIndex; index <= stopIndex; index++) {
      itemStatusMap[index] = LOADING;
    }
    return new Promise((resolve) =>
      setTimeout(() => {
        for (let index = startIndex; index <= stopIndex; index++) {
          itemStatusMap[index] = LOADED;
        }
        // resolve();
      }, 2500)
    );
  };
  return (
    <TableWrapper>
      {title}

      <Table
        style={{
          maxHeight,
          marginLeft: "10px",
          width: "calc(100vw - 272px)",
        }}
      >
        <TableHead>
          {header?.map((column, index) => (
            <TableHeadCell
              key={index}
              style={{
                color: "rgba(0, 0, 0, 0.54)",
                border: "none",
                width:
                  column === "#"
                    ? "70px"
                    : column === "Time"
                    ? "50px"
                    : "initial",
                flex:
                  column === "Time" || column === "#"
                    ? "initial"
                    : column === "Title"
                    ? 0.97
                    : 0.6,
                paddingLeft:
                  column !== "#" && column !== "Title" ? 40 : "initial",
              }}
            >
              {column}
            </TableHeadCell>
          ))}
          <TableHeadCell style={{ flex: 0.5 }}></TableHeadCell>
        </TableHead>

        <div style={{ height: maxHeight }}>
          <AutoSizer>
            {({ height, width }) => (
              <InfiniteLoader
                isItemLoaded={isItemLoaded}
                itemCount={tracks.length}
                loadMoreItems={loadMoreItems}
              >
                {({ onItemsRendered, ref }) => (
                  <List
                    className="List"
                    height={height}
                    itemCount={tracks.length}
                    itemSize={78}
                    onItemsRendered={onItemsRendered}
                    ref={ref}
                    width={width}
                  >
                    {({ index, style }) => (
                      <div style={style}>
                        <TableRow className="tablerow">
                          {header?.map((item, key) => (
                            <Track
                              key={key}
                              row={tracks[index]}
                              item={item}
                              currentIndex={currentIndex}
                              currentTrackId={currentTrackId}
                              isPlaying={isPlaying}
                              index={index}
                              header={header}
                              onPlayTrack={onPlayTrack}
                            />
                          ))}
                          <TableCell
                            style={{
                              flex: 0.4,
                            }}
                          >
                            <CellWrapper>
                              <ContextMenu
                                track={tracks[index]}
                                onPlayNext={onPlayNext}
                                onCreatePlaylist={onCreatePlaylist}
                                recentPlaylists={recentPlaylists}
                                onAddTrackToPlaylist={onAddTrackToPlaylist}
                              />
                            </CellWrapper>
                          </TableCell>
                        </TableRow>
                      </div>
                    )}
                  </List>
                )}
              </InfiniteLoader>
            )}
          </AutoSizer>
        </div>
      </Table>

      {/*
      <TableBuilder
        data={tracks}
        overrides={{
          Root: {
            style: {
              maxHeight,
              paddingLeft: "10px",
            },
          },
          TableHeadCell: {
            style: ({ $col }) => {
              return {
                width:
                  $col.header === "#"
                    ? "10px"
                    : $col.header === "Time"
                    ? "98px"
                    : $col.header === ""
                    ? "50px"
                    : $col.header === "Title"
                    ? "calc(100% - 200px)"
                    : $col.header === "Artist"
                    ? "100px"
                    : "auto",
                outlineWidth: "0px",
                borderBottomColor: "#fff !important",
                color: "rgba(0, 0, 0, 0.542)",
              };
            },
          },
          TableBodyCell: {
            style: ({ $col }) => ({
              outlineWidth: "0px 0px 0px 0px",
              borderBottomColor: "#fff !important",
              overflow: "hidden",
              whiteSpace: "nowrap",
              textOverflow: "ellipsis",
              maxWidth:
                $col.header === "Artist" || $col.header === "Album"
                  ? "120px"
                  : "300px",
            }),
          },
          TableHead: {
            style: () => ({
              outlineWidth: "0px",
              borderBottomColor: "#fff",
            }),
          },
          TableBody: {
            style: () => ({ border: "none" }),
          },
        }}
      >
        {header?.map((item, index) => (
          <TableBuilderColumn key={index} header={item}>
            {(row: any) => {
              const current =
                (item === "Title" || item === "#") &&
                ((currentIndex && currentIndex === row.index) ||
                  (currentTrackId && row.id === currentTrackId)) &&
                isPlaying;
              return (
                <Cell
                  current={current}
                  row={row}
                  item={item}
                  index={index}
                  onPlayTrack={onPlayTrack}
                  isAlbumTracks={header.includes("#")}
                />
              );
            }}
          </TableBuilderColumn>
        ))}
        <TableBuilderColumn header="">
          {(row: any) => (
            <CellWrapper>
              <ContextMenu
                track={row}
                onPlayNext={onPlayNext}
                onCreatePlaylist={onCreatePlaylist}
                recentPlaylists={recentPlaylists}
                onAddTrackToPlaylist={onAddTrackToPlaylist}
              />
            </CellWrapper>
          )}
        </TableBuilderColumn>
      </TableBuilder>
  */}
    </TableWrapper>
  );
};

TracksTable.defaultProps = {
  header: ["Title", "Artist", "Album", "Time"],
  title: <div />,
  maxHeight: "calc(100vh - 250px)",
};

export default TracksTable;
