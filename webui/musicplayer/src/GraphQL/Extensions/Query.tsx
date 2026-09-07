import gql from "graphql-tag";

/**
 * The installed WebAssembly extensions.
 *
 * The filter is applied on the daemon side — it searches the id, name,
 * description, author, topics and capabilities, which is more than the page
 * could reasonably reproduce from the fields it displays.
 */
export const GET_EXTENSIONS = gql`
  query GetExtensions($filter: String) {
    extensions(filter: $filter) {
      id
      name
      version
      author
      description
      homepage
      repository
      license
      logo
      topics
      capabilities
      allowedHosts
      libraryRead
      status
      path
    }
  }
`;
