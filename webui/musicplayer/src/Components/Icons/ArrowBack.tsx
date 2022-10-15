import { FC } from "react";

const ArrowBack: FC = (props) => (
  <svg
    width={20}
    xmlns="http://www.w3.org/2000/svg"
    height={20}
    style={{
      WebkitPrintColorAdjust: "exact",
    }}
    fill="none"
    {...props}
  >
    <g
      style={{
        fill: "#000",
      }}
    >
      <path
        fill="none"
        d="M0 0h20v20H0Z"
        style={{
          fill: "none",
        }}
      />
      <path
        d="M16 9.25H6.873l4.192-4.193L10 4l-6 6 6 6 1.058-1.058-4.185-4.192H16v-1.5Z"
        style={{
          fill: "#000",
        }}
      />
    </g>
  </svg>
);

export default ArrowBack;
