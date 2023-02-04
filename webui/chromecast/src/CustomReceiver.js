import { useContext, useEffect, useState } from "react";
import { CastContext } from "./CastProvider";
import styled from "@emotion/styled";
import Header from "./Components/Header";
import MediaInfo from "./Components/MediaInfo";
import Progress from "./Components/Progress";
import Splash from "./Components/Splash";
import { BehaviorSubject } from "rxjs";
import { css } from "@emotion/react";

const cast = window.cast;

const Separator = styled.div`
  height: calc(100vh - 70px - 54vh);
  width: 100%;
`;

const Container = styled.div`
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
  height: 100vh;
  width: 100%;
  ${(props) =>
    props.cover &&
    css`
      background-image: url(${props.cover});
    `}
  ${(props) =>
    !props.cover &&
    css`
      background-color: #080808;
    `}
`;

const Blur = styled.div`
  background: rgba(0, 0, 0, 0.7);
  /*backdrop-filter: blur(5px);*/
  height: 100vh;
`;

const CustomReceiver = () => {
  const [track, setTrack] = useState({
    artist: "",
    title: "",
    images: [],
  });
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const request$ = new BehaviorSubject(null);
  const { playerManager } = useContext(CastContext);

  useEffect(() => {
    if (playerManager) {
      playerManager.addEventListener(
        cast.framework.events.EventType.MEDIA_STATUS,
        (request) => {
          request$.next({
            ...request,
            eventType: cast.framework.events.EventType.MEDIA_STATUS,
          });
        }
      );

      playerManager.addEventListener(
        cast.framework.events.EventType.TIME_UPDATE,
        (request) => {
          request$.next({
            ...request,
            eventType: cast.framework.events.EventType.TIME_UPDATE,
          });
        }
      );
    }
  }, [playerManager]);

  useEffect(() => {
    const subscription = request$.subscribe((request) => {
      if (
        request &&
        request.mediaStatus &&
        request.mediaStatus.playerState === "PLAYING" &&
        request.eventType === cast.framework.events.EventType.MEDIA_STATUS
      ) {
        setIsLoading(false);
        setTrack(request.mediaStatus.media.metadata);
        setDuration(request.mediaStatus.media.duration);
      }
      if (
        request &&
        request.eventType === cast.framework.events.EventType.TIME_UPDATE
      ) {
        setCurrentTime(request.currentMediaTime);
      }
    });
    return () => subscription.unsubscribe();
  }, []);

  return (
    <>
      {isLoading && <Splash />}
      {!isLoading && (
        <Container cover={track.images[0].url}>
          <Blur>
            <Header />
            <Separator />
            <MediaInfo {...track} />
            <Progress
              currentTime={currentTime * 1000}
              duration={duration * 1000}
            />
          </Blur>
        </Container>
      )}
    </>
  );
};

export default CustomReceiver;
