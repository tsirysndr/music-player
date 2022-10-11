import styled from "@emotion/styled";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const PlayQueue = () => {
  return (
    <Container>
      <Sidebar />
      <div>
        <ControlBar />
        <MainContent title="Play Queue" />
      </div>
    </Container>
  );
};

export default PlayQueue;
