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
              borderWidth: "0px !important",
              borderRadius: "5px !important",
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
            },
          },
        }}
      />
    </>
  );
};

export default Search;
