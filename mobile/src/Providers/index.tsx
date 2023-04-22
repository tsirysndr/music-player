import React, {FC} from 'react';
import {
  ApolloClient,
  createHttpLink,
  InMemoryCache,
  split,
  ApolloProvider,
  ApolloLink,
} from '@apollo/client';
import {WebSocketLink} from '@apollo/client/link/ws';
import {getMainDefinition} from '@apollo/client/utilities';
import {SubscriptionClient} from 'subscriptions-transport-ws';
import Config from 'react-native-config';

const uri = Config.API_URL || 'http://localhost:5053/graphql';

const httpLink = createHttpLink({
  uri,
});

const wsLink = new WebSocketLink(
  new SubscriptionClient(uri.replace('http', 'ws')),
);

const splitLink = split(
  ({query}) => {
    const definition = getMainDefinition(query);
    return (
      definition.kind === 'OperationDefinition' &&
      definition.operation === 'subscription'
    );
  },
  wsLink,
  httpLink,
);

const link: ApolloLink = splitLink;

const client = new ApolloClient({
  link,
  cache: new InMemoryCache(),
});

const Providers: FC<{children: React.ReactNode}> = ({children}) => {
  console.log(Config.API_URL);
  return <ApolloProvider client={client}>{children}</ApolloProvider>;
};

export default Providers;
