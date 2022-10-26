import styled from "@emotion/styled";
import { TableBuilder, TableBuilderColumn } from "baseui/table-semantic";
import _ from "lodash";
import { FC } from "react";
import ContextMenu from "../ContextMenu";
import Speaker from "../Icons/Speaker";

const TableWrapper = styled.div`
  margin-top: 31px;
`;

export type TracksTableProps = {
  tracks: any[];
  header?: string[];
  title?: JSX.Element;
  currentIndex?: number;
  currentTrackId?: string;
  isPlaying?: boolean;
  maxHeight?: string;
};

const TracksTable: FC<TracksTableProps> = ({
  tracks,
  header,
  title,
  currentIndex,
  currentTrackId,
  isPlaying,
  maxHeight,
}) => {
  return (
    <TableWrapper>
      {title}
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
                item === "Title" &&
                ((currentIndex && currentIndex === row.index) ||
                  (currentTrackId && row.id === currentTrackId)) &&
                isPlaying;
              return (
                <>
                  {current && (
                    <div>
                      <div
                        style={{
                          position: "absolute",
                          left: -4,
                          marginTop: -1,
                        }}
                      >
                        <Speaker color="#ab28fc" />
                      </div>
                      <div>{_.get(row, _.toLower(item), "")}</div>
                    </div>
                  )}
                  {!current && <div>{_.get(row, _.toLower(item), "")}</div>}
                </>
              );
            }}
          </TableBuilderColumn>
        ))}
        <TableBuilderColumn header="">
          {(row: any) => <ContextMenu track={row} />}
        </TableBuilderColumn>
      </TableBuilder>
    </TableWrapper>
  );
};

TracksTable.defaultProps = {
  header: ["Title", "Artist", "Album", "Time"],
  title: <div />,
  maxHeight: "calc(100vh - 250px)",
};

export default TracksTable;
