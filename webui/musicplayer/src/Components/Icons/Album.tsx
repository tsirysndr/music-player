import { FC } from "react";

export type AlbumProps = {
  size?: number;
  color?: string;
  width?: number;
  height?: number;
};

const Album: FC<AlbumProps> = (props) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox={`0 0 ${props.size} ${props.size}`}
    {...props}
  >
    <path
      d="M20 5.63A10.2 10.2 0 0 0 4 18.37a10.13 10.13 0 0 0 6.84 3.77 10.42 10.42 0 0 0 1.16.06 10.2 10.2 0 0 0 8-16.57Zm.65 7.37A8.7 8.7 0 1 1 12 3.3a8.79 8.79 0 0 1 1 .05A8.7 8.7 0 0 1 20.65 13Z"
      fill={props.color}
    />
    <path
      d="M12 9a3 3 0 1 0 3 3 3 3 0 0 0-3-3Zm0 4.5a1.5 1.5 0 1 1 1.5-1.5 1.5 1.5 0 0 1-1.5 1.5Z"
      fill={props.color}
    />
  </svg>
);

Album.defaultProps = {
  size: 24,
  color: "#000",
};

export default Album;
