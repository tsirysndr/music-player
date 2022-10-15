import * as React from "react";

export type AddProps = {
  size?: number;
  color?: string;
};

const Add: React.FC<AddProps> = (props) => (
  <svg
    width={props.size}
    xmlns="http://www.w3.org/2000/svg"
    height={props.size}
    style={{
      WebkitPrintColorAdjust: "exact",
    }}
    fill="none"
    {...props}
  >
    <g
      className="ionicon"
      style={{
        fill: props.color,
      }}
    >
      <path
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M11 5v12m6-6H5"
        style={{
          fill: "none",
        }}
      />
      <path
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M11 5v12m6-6H5"
        style={{
          fill: "none",
          strokeWidth: 2,
          stroke: props.color,
        }}
        className="stroke-shape"
      />
    </g>
  </svg>
);

Add.defaultProps = {
  size: 17,
  color: "#000",
};

export default Add;
