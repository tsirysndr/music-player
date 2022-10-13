import styled from "@emotion/styled";
import { Table } from "baseui/table-semantic";
import { FC } from "react";

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
    </TableWrapper>
  );
};

TracksTable.defaultProps = {
  header: ["Title", "Artist", "Album", "Time"],
  title: <div />,
};

export default TracksTable;
