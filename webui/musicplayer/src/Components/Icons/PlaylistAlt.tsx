import * as React from "react";

export type PlaylistProps = {
  size?: number;
  color?: string;
  style?: React.CSSProperties;
};

const Playlist: React.FC<PlaylistProps> = (props) => {
  const { size, color } = props;
  return (
    <svg
      viewBox={`0 0 ${size!} ${size!}`}
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      {...props}
    >
      <g stroke={color!} strokeMiterlimit={10}>
        <path
          d="M14.1 29.3a2.8 2.8 0 1 0 0-5.6 2.8 2.8 0 0 0 0 5.6Z"
          strokeLinecap="round"
        />
        <path d="M16.9 26.5V12.8M21 24.2h8.3M21 16.9h10.1M21 20.5h9.2" />
      </g>
    </svg>
  );
};

Playlist.defaultProps = {
  size: 48,
  color: "#000",
};

export default Playlist;
