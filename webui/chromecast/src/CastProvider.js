import { createContext, useMemo } from "react";

const cast = window.cast;
const context = cast.framework.CastReceiverContext.getInstance();
context.start();
const _playerManager = context.getPlayerManager();

export const CastContext = createContext({
  playerManager: _playerManager,
});

const CastProvider = ({ children }) => {
  const playerManager = useMemo(() => _playerManager, []);
  return (
    <CastContext.Provider
      value={{
        playerManager,
      }}
    >
      {children}
    </CastContext.Provider>
  );
};

export default CastProvider;
