import { CSSProperties, FC } from "react";

export type DiscProps = {
  size?: number;
  color?: string;
  style?: CSSProperties;
};

const Disc: FC<DiscProps> = ({ size, color, ...props }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    width={size}
    height={size}
    fill={color}
    {...props}
  >
    <path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 18a8 8 0 1 1 8-8 8 8 0 0 1-8 8z" />
    <path d="M12 8a4 4 0 1 0 4 4 4 4 0 0 0-4-4zm0 6a2 2 0 1 1 2-2 2 2 0 0 1-2 2z" />
  </svg>
);

Disc.defaultProps = {
  size: 24,
  color: "#000",
};

export default Disc;
