export const getApiUrl = (): string =>
  import.meta.env.DEV
    ? import.meta.env.VITE_API_URL || "http://localhost:3001/graphql"
    : // eslint-disable-next-line no-restricted-globals
      `${origin}/graphql`;

export const getWsUrl = (): string => getApiUrl().replace("http", "ws");

type GraphQLResponse<TData> = {
  data?: TData;
  errors?: { message: string }[];
};

/**
 * GraphQL fetcher used by the hooks generated with
 * @graphql-codegen/typescript-react-query.
 *
 * GraphQL answers 200 even when the operation failed, so the errors array is
 * the only thing that says so — hence the throw rather than a status check.
 */
export const fetcher = <TData, TVariables>(
  query: string,
  variables?: TVariables,
  options?: RequestInit["headers"]
) => {
  return async (): Promise<TData> => {
    const res = await fetch(getApiUrl(), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...options,
      },
      body: JSON.stringify({ query, variables }),
    });
    const response: GraphQLResponse<TData> = await res.json();

    if (response.errors && response.errors.length > 0) {
      throw new Error(response.errors[0].message);
    }

    return response.data as TData;
  };
};
