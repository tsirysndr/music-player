import * as React from "react";

export type HeartProps = {
  size?: number;
  color?: string;
  width?: number;
  height?: number;
};

const Heart: React.FC<HeartProps> = (props) => (
  <svg
    width={props.size}
    xmlns="http://www.w3.org/2000/svg"
    height={props.size}
    style={{
      WebkitPrintColorAdjust: "exact",
      paddingTop: 1,
    }}
    fill="none"
    {...props}
  >
    <g
      style={{
        fill: props.color,
      }}
    >
      <path
        d="M10 17c-.247 0-.488-.071-.692-.203-3.023-1.945-4.332-3.279-5.053-4.113C2.716 10.907 1.98 9.082 2 7.106 2.025 4.842 3.941 3 6.273 3c1.695 0 2.869.905 3.553 1.659a.236.236 0 0 0 .348 0C10.858 3.905 12.032 3 13.727 3 16.059 3 17.975 4.842 18 7.107c.02 1.976-.717 3.801-2.255 5.578-.721.834-2.03 2.167-5.053 4.112A1.275 1.275 0 0 1 10 17Z"
        style={{
          fill: props.color,
          fillOpacity: 1,
        }}
        className="ionicon"
      />
    </g>
  </svg>
);

Heart.defaultProps = {
  size: 20,
  color: "#ab28fc",
};

export default Heart;
