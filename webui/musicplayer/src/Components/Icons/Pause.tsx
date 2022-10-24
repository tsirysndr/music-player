import * as React from "react";

export type PauseProps = {
  color?: string;
  small?: boolean;
};

const Pause: React.FC<PauseProps> = (props) => (
  <svg
    width={props.small ? 20 : 32}
    xmlns="http://www.w3.org/2000/svg"
    height={props.small ? 20 : 32}
    style={{
      WebkitPrintColorAdjust: "exact",
    }}
    viewBox="0 0 512 512"
    {...props}
  >
    <title>{"Pause"}</title>
    <path
      d="M208 432h-48a16 16 0 0 1-16-16V96a16 16 0 0 1 16-16h48a16 16 0 0 1 16 16v320a16 16 0 0 1-16 16zm144 0h-48a16 16 0 0 1-16-16V96a16 16 0 0 1 16-16h48a16 16 0 0 1 16 16v320a16 16 0 0 1-16 16z"
      style={{
        fill: props.color,
      }}
    />
  </svg>
);

Pause.defaultProps = {
  color: "#000",
};

export default Pause;
