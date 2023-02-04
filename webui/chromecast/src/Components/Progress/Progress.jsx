import styled from "@emotion/styled";
import { ProgressBar } from "baseui/progress-bar";
import { useTimeFormat } from "../../Hooks/useFormat";

const Container = styled.div`
  padding-left: calc(5vw - 10px);
  padding-right: calc(5vw - 10px);
  padding-top: 20px;
`;

const Time = styled.div``;

const TimeContainer = styled.div`
  display: flex;
  justify-content: space-between;
  padding-left: 10px;
  padding-right: 10px;
`;
const Progress = ({ currentTime, duration }) => {
  const { formatTime } = useTimeFormat();
  return (
    <Container>
      <ProgressBar
        value={(currentTime / duration) * 100}
        overrides={{
          BarProgress: {
            style: () => ({
              backgroundColor: "rgb(171, 40, 252)",
            }),
          },
        }}
      />
      <TimeContainer>
        <Time>{formatTime(currentTime)}</Time>
        <Time>{formatTime(duration)}</Time>
      </TimeContainer>
    </Container>
  );
};

Progress.defaultProps = {
  currentTime: 0,
  duration: 0,
};

export default Progress;
