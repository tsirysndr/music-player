import styled from "@emotion/styled";
import Add from "../Icons/Add";

const Title = styled.div`
  font-size: 16px;
  font-family: RockfordSansBold;
  margin-bottom: 10px;
`;

const Container = styled.div`
  margin-left: 10px;
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

const Create = styled.div`
  height: 45px;
  width: 183px;
  display: flex;
  flex-direction: row;
  align-items: center;
  font-size: 14px;
  cursor: pointer;
`;

const Plus = styled.div`
  width: 28px;
`;

const Playlists = () => {
  return (
    <Container>
      <Title>Playlists</Title>
      <Create>
        <Plus>
          <Add />
        </Plus>
        <span>Create ...</span>
      </Create>
      <Item>New South</Item>
    </Container>
  );
};

export default Playlists;
