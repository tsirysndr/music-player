import styled from "@emotion/styled";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Tracks = () => {
  return (
    <Container>
      <Sidebar />
      <div>
        <ControlBar />
        <MainContent title="Tracks" />
      </div>
    </Container>
  );
};

export default Tracks;
