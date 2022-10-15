import * as React from "react";

export type PlayProps = {
  color?: string;
  small?: boolean;
};

const Play: React.FC<PlayProps> = (props) => (
  <svg
    width={props.small ? 20 : 32}
    xmlns="http://www.w3.org/2000/svg"
    height={props.small ? 20 : 32}
    style={{
      WebkitPrintColorAdjust: "exact",
    }}
    fill="none"
    {...props}
  >
    {!props.small && (
      <path
        d="M10.657 27a2.842 2.842 0 0 1-1.258-.292C8.536 26.283 8 25.458 8 24.563V6.438c0-.899.536-1.721 1.399-2.146a2.856 2.856 0 0 1 2.571.028l17.817 9.273c.755.411 1.213 1.131 1.213 1.906 0 .774-.458 1.495-1.213 1.906l-17.82 9.275a2.846 2.846 0 0 1-1.31.32Z"
        style={{
          fill: props.color,
        }}
      />
    )}
    {props.small && (
      <path
        d="M7.386 16c-.23 0-.456-.057-.656-.165-.45-.24-.73-.706-.73-1.213V4.378a1.387 1.387 0 0 1 2.071-1.197l9.296 5.241c.394.232.633.64.633 1.077 0 .438-.239.845-.633 1.078L8.07 15.819a1.39 1.39 0 0 1-.684.181Z"
        style={{
          fill: props.color,
        }}
        className="ionicon"
      />
    )}
  </svg>
);

Play.defaultProps = {
  color: "#000",
  small: false,
};

export default Play;
