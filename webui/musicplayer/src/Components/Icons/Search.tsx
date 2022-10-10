import * as React from "react";

const Search: React.FC = (props) => (
  <svg
    width={20}
    height={20}
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    data-test="icon-ActionActiveSearch"
    className="tidal-ui__icon css-11jp7h2"
    {...props}
  >
    <path
      fillRule="evenodd"
      clipRule="evenodd"
      d="M4 9.5a5.5 5.5 0 1 1 11 0 5.5 5.5 0 0 1-11 0ZM9.5 2a7.5 7.5 0 1 0 4.549 13.463l2.744 2.744a1 1 0 0 0 1.414-1.414l-2.744-2.744A7.5 7.5 0 0 0 9.5 2Z"
      fill="#ACAAAA"
    />
  </svg>
);

export default Search;
