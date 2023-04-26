import React, {FC} from 'react';
import ContextMenu from './ContextMenu';
import {useRecoilState} from 'recoil';
import {contextMenuState} from './ContextMenuState';

const ContextMenuWithData: FC = () => {
  const [contextMenu, setContextMenu] = useRecoilState(contextMenuState);
  const onClose = () => {
    setContextMenu({
      ...contextMenu,
      visible: false,
    });
  };
  return <ContextMenu isVisible={contextMenu.visible} onClose={onClose} />;
};

export default ContextMenuWithData;
