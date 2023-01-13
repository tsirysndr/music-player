import * as React from "react";

export type PreviousProps = {
  color?: string;
};

const Previous: React.FC<PreviousProps> = (props) => (
  <svg
    width={18}
    xmlns="http://www.w3.org/2000/svg"
    height={18}
    style={{
      WebkitPrintColorAdjust: "exact",
    }}
    fill="none"
    {...props}
  >
    <path
      d="M3.55 2c.304 0 .55.243.55.542V7.16l8.001-4.716a1.224 1.224 0 0 1 1.23-.015c.413.23.669.677.669 1.162v9.818c0 .486-.256.932-.669 1.162-.382.216-.853.21-1.23-.015L4.1 9.84v4.618a.546.546 0 0 1-.55.542.546.546 0 0 1-.55-.542V2.542C3 2.243 3.246 2 3.55 2Z"
      className="ionicon"
      style={{
        fill: props.color,
      }}
    />
  </svg>
);

Previous.defaultProps = {
  color: "#000",
};

export default Previous;
