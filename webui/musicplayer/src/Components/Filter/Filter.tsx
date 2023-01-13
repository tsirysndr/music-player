import { useTheme } from "@emotion/react";
import { Input } from "baseui/input";
import { FC } from "react";
import Search from "../Icons/Search";

export type FilterProps = {
  placeholder?: string;
};

const Filter: FC<FilterProps> = ({ placeholder }) => {
  const theme = useTheme();
  return (
    <>
      <Input
        startEnhancer={<Search />}
        placeholder={placeholder}
        overrides={{
          Root: {
            style: {
              height: "36px",
              width: "222px",
              borderTopWidth: "1px !important",
              borderBottomWidth: "1px !important",
              borderLeftWidth: "1px !important",
              borderRightWidth: "1px !important",
              borderTopColor: "rgba(82, 82, 82, 0.0625) !important",
              borderBottomColor: "rgba(82, 82, 82, 0.0625) !important",
              borderLeftColor: "rgba(82, 82, 82, 0.0625) !important",
              borderRightColor: "rgba(82, 82, 82, 0.0625) !important",
              borderTopLeftRadius: "18px !important",
              borderTopRightRadius: "18px !important",
              borderBottomLeftRadius: "18px !important",
              borderBottomRightRadius: "18px !important",
              backgroundColor: theme.colors.searchBackground,
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
            },
          },
          StartEnhancer: {
            style: {
              backgroundColor: theme.colors.searchBackground,
              paddingTop: "0px",
              paddingBottom: "0px",
              paddingLeft: "0px",
              paddingRight: "0px",
            },
          },
        }}
      />
    </>
  );
};

Filter.defaultProps = {
  placeholder: "Filter",
};

export default Filter;
