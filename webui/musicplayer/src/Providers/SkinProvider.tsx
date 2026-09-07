import { useAtom } from "jotai";
import { createContext, FC, useCallback, useContext, useEffect } from "react";
import { SKINS, SkinId, SKIN_STORAGE_KEY, skinAtom } from "../State";

export type SkinContextType = {
  skin: SkinId;
  /** The display name the sidebar shows, e.g. "Late Night". */
  skinName: string;
  setSkin: (skin: SkinId) => void;
  /** Step to the next skin — what the desktop's `s` shortcut does. */
  cycleSkin: () => void;
};

export const SkinContext = createContext<SkinContextType>({
  skin: SKINS[0].id,
  skinName: SKINS[0].name,
  setSkin: () => {},
  cycleSkin: () => {},
});

export const useSkin = () => useContext(SkinContext);

export type SkinProviderProps = {
  children: React.ReactNode;
};

/**
 * Puts the chosen skin on `<html data-skin="…">`, which is what every token in
 * `styles/skins.css` hangs off. Nothing else needs to know the skin: components
 * read `--accent` and friends, and they change underneath them.
 */
const SkinProvider: FC<SkinProviderProps> = ({ children }) => {
  const [skin, setSkin] = useAtom(skinAtom);

  useEffect(() => {
    document.documentElement.dataset.skin = skin;
    window.localStorage.setItem(SKIN_STORAGE_KEY, skin);
  }, [skin]);

  const cycleSkin = useCallback(() => {
    setSkin((current) => {
      const index = SKINS.findIndex((entry) => entry.id === current);
      return SKINS[(index + 1) % SKINS.length].id;
    });
  }, [setSkin]);

  const skinName =
    SKINS.find((entry) => entry.id === skin)?.name ?? SKINS[0].name;

  return (
    <SkinContext.Provider value={{ skin, skinName, setSkin, cycleSkin }}>
      {children}
    </SkinContext.Provider>
  );
};

export default SkinProvider;
