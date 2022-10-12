import { FC } from "react";

export type TrackProps = {
  size?: number;
  color?: string;
  width?: number;
  height?: number;
};

const Track: FC<TrackProps> = (props) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox={`0 0 ${props.size} ${props.size}`}
    {...props}
  >
    <path
      d="M8.1 4.65v11.26a3.45 3.45 0 1 0 1.5 2.84V5.85l10.2-2.36v10.62A3.45 3.45 0 1 0 21.3 17V1.61Zm-2 16a2 2 0 1 1 2-2 2 2 0 0 1-1.95 2.05Zm11.7-1.8a1.95 1.95 0 1 1 2-1.85 2 2 0 0 1-1.95 1.9Z"
      fill={props.color}
    />
  </svg>
);

Track.defaultProps = {
  size: 24,
  color: "#000",
};

export default Track;
