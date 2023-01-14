import * as React from "react";

export type AddAltProps = {
  color?: string;
};

const SvgComponent: React.FC<AddAltProps> = (props) => (
  <svg
    width={17}
    xmlns="http://www.w3.org/2000/svg"
    height={17}
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
        d="M9.5 5v9M14 9.5H5"
        style={{
          fill: "none",
        }}
      />
      <path
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M9.5 5v9M14 9.5H5"
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

SvgComponent.defaultProps = {
  color: "#000",
};

export default SvgComponent;
