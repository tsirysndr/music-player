import styled from "@emotion/styled";
import { FC } from "react";
import { useLocation } from "react-router-dom";

const Title = styled.div`
  font-size: 16px;
  font-family: RockfordSansBold;
  margin-top: 35px;
  margin-bottom: 5px;
  color: ${(props) => props.theme.colors.text};
`;

const Item = styled.div<{ active?: boolean }>`
  cursor: pointer;
  height: 45px;
  width: 183px;
  display: flex;
  align-items: center;
  font-size: 14px;
  color: ${(props) => props.theme.colors.text};
  ${(props) => (props.active ? "color: #ab28fc;" : "initial")}
`;

const Container = styled.div`
  margin-bottom: 30px;
  margin-left: 10px;
`;

export type LibraryProps = {
  active?: string;
  onClickLibraryItem: (item: string) => void;
};

const Library: FC<LibraryProps> = ({ active, onClickLibraryItem }) => {
  const { pathname } = useLocation();
  return (
    <Container>
      <Title>Library</Title>
      <Item
        active={pathname.includes("/tracks")}
        onClick={() => onClickLibraryItem("tracks")}
      >
        Tracks
      </Item>
      <Item
        active={pathname.includes("/albums")}
        onClick={() => onClickLibraryItem("albums")}
      >
        Albums
      </Item>
      <Item
        active={pathname.includes("/artists")}
        onClick={() => onClickLibraryItem("artists")}
      >
        Artists
      </Item>
    </Container>
  );
};

Library.defaultProps = {
  active: "tracks",
};

export default Library;
