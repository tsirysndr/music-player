import * as React from "react";

export type RepeatProps = {
  color?: string;
};

const Repeat: React.FC<RepeatProps> = (props) => (
  <svg
    width={16}
    height={16}
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    data-test="icon-PlaybackInactiveRepeat"
    viewBox="0 0 24 24"
    {...props}
  >
    <path
      fillRule="evenodd"
      clipRule="evenodd"
      d="M16.293 1.293a1 1 0 0 1 1.414 0l3 3a1 1 0 0 1 0 1.414l-3 3a1 1 0 1 1-1.414-1.414L17.586 6H8.5c-3.298 0-6 2.702-6 6a1 1 0 1 1-2 0c0-4.402 3.598-8 8-8h9.086l-1.293-1.293a1 1 0 0 1 0-1.414ZM23.5 12a1 1 0 1 0-2 0c0 3.298-2.702 6-6 6H6.414l1.293-1.293a1 1 0 1 0-1.414-1.414l-3 3a1 1 0 0 0 0 1.414l3 3a1 1 0 0 0 1.414-1.414L6.414 20H15.5c4.402 0 8-3.598 8-8Z"
      fill={props.color}
    />
  </svg>
);

Repeat.defaultProps = {
  color: "#000",
};

export default Repeat;
