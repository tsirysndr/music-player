import { createContext, useContext, type ReactNode } from "react";

/**
 * Whether the connected library can report a key and a tempo.
 *
 * Ambient rather than a prop: it is one fact about the whole session, and every
 * track list in the app would otherwise have to accept it and pass it on twice
 * — to its header and to each of its rows — for something none of them decide.
 *
 * Defaults to `false`, so a component rendered without a provider (a story, a
 * test) shows the columns it can actually fill.
 */
const LibraryAnalysisContext = createContext(false);

export const LibraryAnalysisProvider = ({
  value,
  children,
}: {
  value: boolean;
  children: ReactNode;
}) => (
  <LibraryAnalysisContext.Provider value={value}>
    {children}
  </LibraryAnalysisContext.Provider>
);

/** Whether to show the key and tempo columns. */
export const useLibraryAnalysis = () => useContext(LibraryAnalysisContext);
