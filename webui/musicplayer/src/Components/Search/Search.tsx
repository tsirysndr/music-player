import { Input } from "baseui/input";

const Search = () => {
  return (
    <>
      <Input
        placeholder="Search"
        overrides={{
          Root: {
            style: {
              height: "36px",
              width: "222px",
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

export default Search;
