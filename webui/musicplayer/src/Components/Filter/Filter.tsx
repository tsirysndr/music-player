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
              borderWidth: "1px !important",
              borderColor: "rgba(82, 82, 82, 0.0625) !important",
              borderRadius: "18px !important",
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
              padding: "0px",
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
