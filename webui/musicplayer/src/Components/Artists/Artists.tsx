import styled from "@emotion/styled";
import ControlBar from "../ControlBar";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Artists = () => {
  return (
    <Container>
      <Sidebar />
      <div>
        <ControlBar />
      </div>
    </Container>
  );
};

export default Artists;
