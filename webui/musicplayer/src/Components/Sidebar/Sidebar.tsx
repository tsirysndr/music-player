import styled from "@emotion/styled";
import Library from "../Library";
import Playlists from "../Playlists";
import Search from "../Search";

const Container = styled.div`
  height: 100vh;
  padding-top: 30px;
  padding-left: 30px;
  padding-right: 20px;
  min-width: 222px;
  overflow-y: auto;
`;

const Sidebar = () => {
  return (
    <Container>
      <Search />
      <Library />
      <Playlists />
    </Container>
  );
};

export default Sidebar;
