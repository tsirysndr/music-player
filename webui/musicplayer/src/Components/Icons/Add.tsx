import * as React from "react";

const Add: React.FC = (props) => (
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
        fill: "#000",
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
          stroke: "#000",
        }}
        className="stroke-shape"
      />
    </g>
  </svg>
);

export default Add;
