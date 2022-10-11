import styled from "@emotion/styled";
import ControlBar from "../ControlBar";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

const Artists = () => {
  return (
    <Container>
      <Sidebar />
      <Content>
        <ControlBar />
      </Content>
    </Container>
  );
};

export default Artists;
