import { createContext, useState, FC } from "react";
import { ThemeProvider as EmotionThemeProvider } from "@emotion/react";
import {
  BaseUIDarkTheme,
  BaseUILightTheme,
  DarkTheme,
  LightTheme,
} from "../Theme";
import { BaseProvider } from "baseui";

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
  const [theme, setTheme] = useState<Theme>("light");
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
