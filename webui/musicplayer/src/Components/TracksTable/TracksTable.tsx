import styled from "@emotion/styled";
import { TableBuilder, TableBuilderColumn } from "baseui/table-semantic";
import _ from "lodash";
import { FC } from "react";
import ContextMenu from "../ContextMenu";

const TableWrapper = styled.div`
  margin-top: 31px;
`;

export type TracksTableProps = {
  tracks: any[];
  header?: string[];
  title?: JSX.Element;
};

const TracksTable: FC<TracksTableProps> = ({ tracks, header, title }) => {
  return (
    <TableWrapper>
      {title}
      {/*
        <Table
        columns={header}
        data={tracks}
        overrides={{
          TableHeadCell: {
            style: ({ $col }) => {
              return {
                width:
                  $col === "#" ? "10px" : $col === "Time" ? "98px" : "intial",
                outline: `#fff solid`,
                borderBottomColor: "#fff !important",
                color: "rgba(0, 0, 0, 0.542)",
              };
            },
          },
          TableBodyCell: {
            style: () => ({
              outline: `#fff solid`,
              backgroundColor: "#fff",
            }),
          },
          TableHead: {
            style: () => ({
              outline: `#fff solid`,
              borderBottomColor: "#fff",
            }),
          },
          TableBody: {
            style: () => ({ border: "none", backgroundColor: "#fff" }),
          },
        }}
      />
      */}
      <TableBuilder
        data={tracks}
        overrides={{
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
                    : "intial",
                outline: `#fff solid`,
                borderBottomColor: "#fff !important",
                color: "rgba(0, 0, 0, 0.542)",
              };
            },
          },
          TableBodyCell: {
            style: () => ({
              outline: `#fff solid`,
              backgroundColor: "#fff",
            }),
          },
          TableHead: {
            style: () => ({
              outline: `#fff solid`,
              borderBottomColor: "#fff",
            }),
          },
          TableBody: {
            style: () => ({ border: "none", backgroundColor: "#fff" }),
          },
        }}
      >
        {header?.map((item) => (
          <TableBuilderColumn header={item}>
            {(row: any) => <div>{_.get(row, _.toLower(item), "")}</div>}
          </TableBuilderColumn>
        ))}
        <TableBuilderColumn header="">
          {() => <ContextMenu />}
        </TableBuilderColumn>
      </TableBuilder>
    </TableWrapper>
  );
};

TracksTable.defaultProps = {
  header: ["Title", "Artist", "Album", "Time"],
  title: <div />,
};

export default TracksTable;
