import styled from "@emotion/styled";
import { FC, ReactNode } from "react";
import Filter from "../Filter";
import { Table } from "baseui/table-semantic";

const Title = styled.div`
  font-family: RockfordSansBold;
  font-size: 16px;
  margin-top: 30px;
  margin-bottom: 36px;
`;

const Header = styled.div`
  padding-left: 16px;
`;

export type MainContentProps = {
  title: string;
  placeholder?: string;
  children?: ReactNode;
};

const MainContent: FC<MainContentProps> = ({
  title,
  placeholder,
  children,
}) => {
  return (
    <div>
      <Header>
        <Title>{title}</Title>
        <Filter placeholder={placeholder} />
      </Header>
      {children}
    </div>
  );
};

MainContent.defaultProps = {
  placeholder: "Filter Tracks",
};

export default MainContent;
