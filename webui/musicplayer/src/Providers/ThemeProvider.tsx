import { createContext, FC } from "react";
import { ThemeProvider as EmotionThemeProvider } from "@emotion/react";
import { useAtom } from "jotai";
import {
  BaseUIDarkTheme,
  BaseUILightTheme,
  DarkTheme,
  LightTheme,
} from "../Theme";
import { BaseProvider } from "baseui";
import { themeAtom } from "../State";

export type Theme = "light" | "dark";

export type ThemeContextType = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
};

export const ThemeContext = createContext<ThemeContextType>({
  theme: "light",
  setTheme: (theme: Theme) => {},
});

export type ThemeProviderProps = {
  children: React.ReactNode;
};

const ThemeProvider: FC<ThemeProviderProps> = ({ children }) => {
  // the theme now lives in a jotai atom, the ThemeContext is kept so
  // existing consumers keep working unchanged
  const [theme, setTheme] = useAtom(themeAtom);
  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      <EmotionThemeProvider theme={theme === "dark" ? DarkTheme : LightTheme}>
        <BaseProvider
          theme={theme === "dark" ? BaseUIDarkTheme : BaseUILightTheme}
        >
          {children}
        </BaseProvider>
      </EmotionThemeProvider>
    </ThemeContext.Provider>
  );
};

export default ThemeProvider;
