import React, {FC, useEffect} from 'react';
import {TouchableWithoutFeedback} from 'react-native';
import Octicons from 'react-native-vector-icons/Octicons';
import MaterialIcons from 'react-native-vector-icons/MaterialIcons';
import styled from '@emotion/native';
import {Controller, useForm} from 'react-hook-form';

const Container = styled.View`
  height: 60px;
  display: flex;
  flex-direction: row;
  align-items: center;
  background-color: #161616;
`;

const SearchInput = styled.TextInput`
  font-family: Gilroy-Medium;
  font-size: 16px;
  flex: 1;
`;

const IconWrapper = styled.View`
  height: 60px;
  width: 60px;
  align-items: center;
  justify-content: center;
`;

export type SearchBarProps = {
  query: string;
  onChange: (value: string) => void;
};

const SearchBar: FC<SearchBarProps> = props => {
  const {control, watch, reset} = useForm({
    defaultValues: {
      search: '',
    },
  });

  useEffect(() => {
    const subscription = watch((value, {name, type}) => {
      if (type === 'change') {
        props.onChange(value[name!] as string);
      }
    });
    return () => subscription.unsubscribe();
  }, [props, watch]);

  const onReset = () => {
    reset();
    props.onChange('');
  };

  return (
    <Container>
      <IconWrapper>
        <Octicons name="search" size={24} color="#bdb9b9" />
      </IconWrapper>
      <Controller
        control={control}
        render={({field: {onChange, onBlur, value}}) => (
          <SearchInput
            onChangeText={onChange}
            onBlur={onBlur}
            value={value}
            cursorColor={'#fff'}
            placeholder="Search"
          />
        )}
        name="search"
        defaultValue=""
      />
      {props.query.length > 0 && (
        <TouchableWithoutFeedback onPress={onReset}>
          <IconWrapper>
            <MaterialIcons name="close" size={24} color="#bdb9b9" />
          </IconWrapper>
        </TouchableWithoutFeedback>
      )}
    </Container>
  );
};

export default SearchBar;
