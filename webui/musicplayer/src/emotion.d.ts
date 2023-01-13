import "@emotion/react";

declare module "@emotion/react" {
  export interface Theme {
    colors: {
      text: string;
      background: string;
      icon: string;
      searchBackground: string;
      secondaryBackground: string;
      secondaryText: string;
      backButton: string;
      currentTrackBorder: string;
      popoverBackground: string;
      separator: string;
      hover: string;
    };
  }
}
