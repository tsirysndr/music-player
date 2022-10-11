import styled from "@emotion/styled";
import { Table } from "baseui/table-semantic";
import { FC } from "react";

const TableWrapper = styled.div`
  margin-top: 31px;
`;

const TracksTable: FC = () => {
  return (
    <TableWrapper>
      <Table
        columns={["Title", "Album", "Artist", "Time"]}
        data={[
          ["Otherside", "Red Hot Chilli Peppers", "Californication", "4:15"],
          [
            "Road Trippin'",
            "Red Hot Chilli Peppers",
            "Californication",
            "3:25",
          ],
        ]}
        overrides={{
          TableHeadCell: {
            style: () => ({
              outline: `#fff solid`,
              borderBottomColor: "#fff !important",
              color: "rgba(0, 0, 0, 0.542)",
            }),
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

export default TracksTable;
