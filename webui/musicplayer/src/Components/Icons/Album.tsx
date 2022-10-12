import { FC } from "react";

const Album: FC = (props) => (
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
    <path d="M20 5.63A10.2 10.2 0 0 0 4 18.37a10.13 10.13 0 0 0 6.84 3.77 10.42 10.42 0 0 0 1.16.06 10.2 10.2 0 0 0 8-16.57Zm.65 7.37A8.7 8.7 0 1 1 12 3.3a8.79 8.79 0 0 1 1 .05A8.7 8.7 0 0 1 20.65 13Z" />
    <path d="M12 9a3 3 0 1 0 3 3 3 3 0 0 0-3-3Zm0 4.5a1.5 1.5 0 1 1 1.5-1.5 1.5 1.5 0 0 1-1.5 1.5Z" />
  </svg>
);

export default Album;
