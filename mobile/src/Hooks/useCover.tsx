import Config from 'react-native-config';

export const useCover = (cover?: string) => {
  return cover?.startsWith('http')
    ? cover
    : `${Config.API_URL?.replace('/graphql', '/covers')}/${cover}`;
};
