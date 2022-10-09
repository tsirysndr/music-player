import styled from "@emotion/styled";
import Library from "../Library";
import Playlists from "../Playlists";
import Search from "../Search";

const Container = styled.div`
  height: 100vh;
  width: 284px;
  padding-left: 28px;
  padding-right: 28px;
  padding-top: 30px;
  overflow-y: scroll;
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
