import { useTheme } from "@emotion/react";
import { Input } from "baseui/input";
import { FC } from "react";

export type SearchProps = {
  onSearch: (query: string) => void;
  height?: string;
  width?: string;
};

const Search: FC<SearchProps> = ({ onSearch, width, height }) => {
  const theme = useTheme();
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
              backgroundColor: theme.colors.searchBackground,
              fontSize: "14px",
              color: theme.colors.text,
            },
          },
          InputContainer: {
            style: {
              backgroundColor: theme.colors.searchBackground,
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
