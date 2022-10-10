import styled from "@emotion/styled";
import Next from "../Icons/Next";
import Play from "../Icons/Play";
import Previous from "../Icons/Previous";
import Repeat from "../Icons/Repeat";
import Shuffle from "../Icons/Shuffle";
import CurrentTrack from "./CurrentTrack";

const Container = styled.div`
  display: flex;
  align-items: center;
  height: 96px;
`;

const Controls = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 32px;
  width: 200px;
`;

const Button = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const ControlBar = () => {
  return (
    <Container>
      <Controls>
        <Button>
          <Shuffle />
        </Button>
        <Button>
          <Previous />
        </Button>
        <Button>
          <Play />
        </Button>
        <Button>
          <Next />
        </Button>
        <Button>
          <Repeat />
        </Button>
      </Controls>
      <CurrentTrack />
    </Container>
  );
};

export default ControlBar;
