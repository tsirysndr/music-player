import * as React from "react";

export type NextProps = {
  color?: string;
};

const Next: React.FC<NextProps> = (props) => (
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
      d="M14.45 2a.546.546 0 0 0-.55.542V7.16L5.899 2.444a1.224 1.224 0 0 0-1.23-.015C4.256 2.659 4 3.105 4 3.591v9.818c0 .486.256.932.669 1.162.382.216.853.21 1.23-.015L13.9 9.84v4.618c0 .299.246.542.55.542.304 0 .55-.243.55-.542V2.542A.546.546 0 0 0 14.45 2Z"
      className="ionicon"
      style={{
        fill: props.color,
      }}
    />
  </svg>
);

Next.defaultProps = {
  color: "#000",
};

export default Next;
