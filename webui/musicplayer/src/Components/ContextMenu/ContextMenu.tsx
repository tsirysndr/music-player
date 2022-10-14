import styled from "@emotion/styled";
import { FC } from "react";
import Add from "../Icons/Add";
import Heart from "../Icons/Heart";
import HeartOutline from "../Icons/HeartOutline";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Separator = styled.div`
  width: 10px;
`;

const Icon = styled.div`
  cursor: pointer;
`;

export type ContextMenuProps = {
  liked?: boolean;
};

const ContextMenu: FC<ContextMenuProps> = ({ liked }) => {
  return (
    <Container>
      <Icon>
        <Add size={24} />
      </Icon>
      <Separator />
      {liked && (
        <Icon>
          <Heart height={24} width={24} />
        </Icon>
      )}
      {!liked && (
        <Icon>
          <HeartOutline height={24} width={24} />
        </Icon>
      )}
    </Container>
  );
};

ContextMenu.defaultProps = {
  liked: false,
};

export default ContextMenu;
