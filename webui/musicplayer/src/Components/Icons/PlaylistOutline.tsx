import { CSSProperties, FC } from "react";

export type PlaylistOutlineProps = {
  size?: number;
  color?: string;
  style?: CSSProperties;
};

const PlaylistOutline: FC<PlaylistOutlineProps> = ({
  size,
  color,
  ...props
}) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    width={size}
    height={size}
    fill="none"
    stroke={color}
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    {...props}
  >
    <path d="M11 17a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
    <path d="M17 17v-13h4" />
    <path d="M13 5h-10" />
    <path d="M3 9l10 0" />
    <path d="M9 13h-6" />
  </svg>
);

PlaylistOutline.defaultProps = {
  size: 24,
  color: "#000",
};

export default PlaylistOutline;
