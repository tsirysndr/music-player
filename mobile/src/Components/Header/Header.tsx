import React, {FC} from 'react';
import styled from '@emotion/native';
import Feather from 'react-native-vector-icons/Feather';
import MaterialIcons from 'react-native-vector-icons/MaterialIcons';

const Container = styled.View`
  flex-direction: row;
  align-items: center;
  height: 50px;
  width: 100%;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 18px;
  flex: 1;
`;

const BackButton = styled.TouchableOpacity`
  width: 50px;
  height: 50px;
  justify-content: center;
  align-items: center;
`;

const SearchButton = styled.TouchableOpacity`
  width: 50px;
  height: 50px;
  justify-content: center;
  align-items: center;
`;

const FilterButton = styled.TouchableOpacity`
  width: 50px;
  height: 50px;
  justify-content: center;
  align-items: center;
`;

const NoTitle = styled.View`
  flex: 1;
`;

const SearchInput = styled.TextInput`
  height: 50px;
  width: 100%;
  color: #fff;
  font-family: Gilroy-Medium;
  font-size: 16px;
`;

export type HeaderProps = {
  onGoBack: () => void;
  onSearch?: () => void;
  onFilter?: () => void;
  onKeyPress?: (text: string) => void;
  title?: string;
  placeholder?: string;
};

const Header: FC<HeaderProps> = props => {
  const {onGoBack, onSearch, onFilter, title, placeholder, onKeyPress} = props;
  return (
    <Container>
      <BackButton onPress={onGoBack}>
        <Feather name="arrow-left" size={24} color="#fff" />
      </BackButton>
      {!title && <NoTitle />}
      {title && <Title numberOfLines={1}>{title}</Title>}
      {onSearch && (
        <SearchButton onPress={onSearch}>
          <Feather name="search" size={24} color="#fff" />
        </SearchButton>
      )}
      {onFilter && (
        <FilterButton onPress={onFilter}>
          <MaterialIcons name="filter-list" size={24} color="#fff" />
        </FilterButton>
      )}
      {placeholder && (
        <SearchInput
          placeholder={placeholder}
          autoFocus={true}
          onChangeText={onKeyPress}
        />
      )}
    </Container>
  );
};

export default Header;
