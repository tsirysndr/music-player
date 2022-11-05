import { Input } from "baseui/input";
import { FC } from "react";
import Search from "../Icons/Search";

export type FilterProps = {
  placeholder?: string;
};

const Filter: FC<FilterProps> = ({ placeholder }) => {
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
              backgroundColor: "#fff",
            },
          },
          Input: {
            style: {
              backgroundColor: "#fff",
              fontSize: "14px",
            },
          },
          InputContainer: {
            style: {
              backgroundColor: "#fff",
            },
          },
          StartEnhancer: {
            style: {
              backgroundColor: "#fff",
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
