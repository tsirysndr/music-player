import { createContext } from "react";

const cast = window.cast;
const context = cast.framework.CastReceiverContext.getInstance();
context.start();
const playerManager = context.getPlayerManager();

export const CastContext = createContext({
  playerManager,
});

const CastProvider = ({ children }) => {
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
