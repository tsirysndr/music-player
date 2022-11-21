import { Input } from "baseui/input";
import { FC } from "react";

export type SearchProps = {
  onSearch: (query: string) => void;
  height?: string;
  width?: string;
};

const Search: FC<SearchProps> = ({ onSearch, width, height }) => {
  const handleKeyPress = (
    event: React.KeyboardEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    if (event.key === "Enter") {
      onSearch(event.currentTarget.value);
    }
  };
  return (
    <>
      <Input
        placeholder="Search"
        onKeyPress={handleKeyPress}
        overrides={{
          Root: {
            style: {
              height,
              width,
              borderTopWidth: "0px !important",
              borderBottomWidth: "0px !important",
              borderLeftWidth: "0px !important",
              borderRightWidth: "0px !important",
              borderTopRadius: "5px !important",
              borderBottomRadius: "5px !important",
              borderLeftRadius: "5px !important",
              borderRightRadius: "5px !important",
            },
          },
          Input: {
            style: {
              backgroundColor: "rgba(177, 178, 181, 0.1)",
              fontSize: "14px",
            },
          },
          InputContainer: {
            style: {
              backgroundColor: "#fff",
              width: "100%",
            },
          },
        }}
      />
    </>
  );
};

Search.defaultProps = {
  height: "36px",
  width: "222px",
};

export default Search;
