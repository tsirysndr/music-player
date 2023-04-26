import React, {FC} from 'react';
import {StyleSheet} from 'react-native';
import Modal from 'react-native-modal';
import styled from '@emotion/native';

const Container = styled.View`
  background-color: #000;
  height: 300px;
`;

export type ContextMenuProps = {
  isVisible: boolean;
  onClose: () => void;
};

const ContextMenu: FC<ContextMenuProps> = ({isVisible, onClose}) => {
  return (
    <Modal
      isVisible={isVisible}
      backdropOpacity={0.03}
      onBackdropPress={onClose}
      onSwipeComplete={onClose}
      swipeThreshold={500}
      swipeDirection={['down']}
      style={styles.modal}>
      <Container></Container>
    </Modal>
  );
};

const styles = StyleSheet.create({
  modal: {
    justifyContent: 'flex-end',
    margin: 0,
  },
});

export default ContextMenu;
