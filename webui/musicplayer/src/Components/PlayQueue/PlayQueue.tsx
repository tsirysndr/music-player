import styled from "@emotion/styled";
import ControlBar from "../ControlBar";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const PlayQueue = () => {
  return (
    <div>
      <Sidebar />
      <div>
        <ControlBar />
      </div>
    </div>
  );
};

export default PlayQueue;
