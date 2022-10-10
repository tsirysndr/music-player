import * as React from "react";

const Play: React.FC = (props) => (
  <svg
    width={32}
    xmlns="http://www.w3.org/2000/svg"
    height={32}
    style={{
      WebkitPrintColorAdjust: "exact",
    }}
    fill="none"
    {...props}
  >
    <path
      d="M10.657 27a2.842 2.842 0 0 1-1.258-.292C8.536 26.283 8 25.458 8 24.563V6.438c0-.899.536-1.721 1.399-2.146a2.856 2.856 0 0 1 2.571.028l17.817 9.273c.755.411 1.213 1.131 1.213 1.906 0 .774-.458 1.495-1.213 1.906l-17.82 9.275a2.846 2.846 0 0 1-1.31.32Z"
      style={{
        fill: "#000",
      }}
    />
  </svg>
);

export default Play;
