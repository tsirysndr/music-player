import styled from "@emotion/styled";

const Container = styled.div`
  height: 70px;
  width: 100%;
  padding-left: 5vw;
  padding-right: 5vw;
  display: flex;
  align-items: center;
`;

const Logo = styled.div`
  font-family: RockfordSansBold;
`;

const Header = () => {
  return (
    <Container>
      <Logo>Music Player</Logo>
    </Container>
  );
};

export default Header;
