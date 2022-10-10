import styled from "@emotion/styled";

const Title = styled.div`
  font-size: 16px;
  font-family: RockfordSansBold;
  margin-top: 35px;
  margin-bottom: 5px;
`;

const Item = styled.div<{ active?: boolean }>`
  cursor: pointer;
  height: 45px;
  width: 183px;
  display: flex;
  align-items: center;
  font-size: 14px;
  ${(props) => (props.active ? "color: #ab28fc;" : "initial")}
`;

const Container = styled.div`
  margin-bottom: 30px;
  margin-left: 10px;
`;

const Library = () => {
  return (
    <Container>
      <Title>Library</Title>
      <Item active>Tracks</Item>
      <Item>Albums</Item>
      <Item>Artists</Item>
      <Item>Play Queue</Item>
    </Container>
  );
};

export default Library;
