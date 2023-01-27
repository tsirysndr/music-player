import styled from "@emotion/styled";
import { Speaker2 } from "@styled-icons/fluentui-system-regular";
import { MusicPlayer } from "@styled-icons/bootstrap";
import { Kodi, Airplayaudio, Chromecast } from "@styled-icons/simple-icons";
import { FC } from "react";

const Container = styled.div`
  display: flex;
  position: relative;
  background: ${(props) => props.theme.colors.tooltip};
  color: #ab28fc;
  justify-content: center;
  align-items: center;
  padding-right: 20px;
  padding-top: 2px;
  padding-bottom: 2px;
  font-size: 12px;
  width: 100%;
`;

const Wrapper = styled.div`
  background-color: ${(props) => props.theme.colors.background};
`;

const IconWrapper = styled.div`
  margin-right: 5px;
  display: flex;
  align-items: center;
  justify-content: center;
`;

export type ListeningOnProps = {
  icon?: "music-player" | "xbmc" | "airplay" | "chromecast";
  deviceName?: string;
};

const ListeningOn: FC<ListeningOnProps> = ({ icon, deviceName }) => {
  return (
    <Wrapper>
      <Container>
        <IconWrapper>
          {
            //<Speaker2 size={15} color="#ab28fc" />
          }
          {icon === "music-player" && <MusicPlayer size={15} color="#ab28fc" />}
          {icon === "xbmc" && <Kodi size={15} color="#ab28fc" />}
          {icon === "airplay" && <Airplayaudio size={15} color={"#ab28fc"} />}
          {icon === "chromecast" && <Chromecast size={15} color={"#ab28fc"} />}
        </IconWrapper>
        <div style={{ marginTop: -3, marginRight: 25 }}>
          Listening on {deviceName}
        </div>
      </Container>
    </Wrapper>
  );
};

ListeningOn.defaultProps = {
  icon: "music-player",
};

export default ListeningOn;
