import { CSSProperties, FC } from "react";

export type MusicNoteProps = {
  size?: number;
  color?: string;
  style?: CSSProperties;
};

const MusicNote: FC<MusicNoteProps> = ({ size, color, ...props }) => (
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
    <path d="M9 18V5l12-2v13" />
    <path d="m9 9 12-2" />
    <circle cx="6" cy="18" r="3" />
    <circle cx="18" cy="16" r="3" />
  </svg>
);

MusicNote.defaultProps = {
  size: 24,
  color: "#000",
};

export default MusicNote;
