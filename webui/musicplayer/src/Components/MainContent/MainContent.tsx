import styled from "@emotion/styled";
import { FC } from "react";
import Filter from "../Filter";
import { Table } from "baseui/table-semantic";

const Title = styled.div`
  font-family: RockfordSansBold;
  font-size: 16px;
  margin-top: 30px;
  margin-bottom: 36px;
`;

const TableWrapper = styled.div`
  margin-top: 31px;
`;

const Header = styled.div`
  padding-left: 16px;
`;

export type MainContentProps = {
  title: string;
};

const MainContent: FC<MainContentProps> = ({ title }) => {
  return (
    <div>
      <Header>
        <Title>{title}</Title>
        <Filter placeholder="Filter Tracks" />
      </Header>

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
    </div>
  );
};

export default MainContent;
