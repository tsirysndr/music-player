import { FC } from "react";
import ThemeProvider from "./ThemeProvider";

export type ProvidersProps = {
  children: React.ReactNode;
};

const Providers: FC<ProvidersProps> = ({ children }) => {
  return <ThemeProvider>{children}</ThemeProvider>;
};

export default Providers;
