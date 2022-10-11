import styled from "@emotion/styled";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
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

const Tracks = () => {
  return (
    <Container>
      <Sidebar />
      <Content>
        <ControlBar />
        <MainContent title="Tracks" />
      </Content>
    </Container>
  );
};

export default Tracks;
