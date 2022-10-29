import * as React from "react";

export type SpeakerProps = {
  size?: number;
  color?: string;
};

const Speaker: React.FC<SpeakerProps> = (props) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    height={props.size}
    width={props.size}
    {...props}
  >
    <path
      d="M4.67 9.61v4.78l2.92.36 5.88 4V5.2l-5.88 4Zm7.8-2.51v9.8L8 13.79l-2.28-.29v-3L8 10.21ZM16.44 6.11 15.9 7a6 6 0 0 1 0 10l.54.84a7 7 0 0 0 0-11.77Z"
      fill={props.color}
    />
    <path
      d="m14.78 7.78-.55.83a4.06 4.06 0 0 1 0 6.77l.55.83a5.05 5.05 0 0 0 0-8.44Z"
      fill={props.color}
    />
  </svg>
);

Speaker.defaultProps = {
  size: 24,
  color: "#000",
};

export default Speaker;
