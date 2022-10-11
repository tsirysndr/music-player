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

const Albums = () => {
  return (
    <Container>
      <Sidebar />
      <Container>
        <ControlBar />
      </Container>
    </Container>
  );
};

export default Albums;
