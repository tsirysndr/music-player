import React, {FC} from 'react';
import {FlatList, TouchableWithoutFeedback} from 'react-native';
import styled, {css} from '@emotion/native';

const LabelWrapper = styled.View<{active?: boolean}>`
  background-color: #202020;
  height: 30px;
  border-radius: 15px;
  padding: 0 15px;
  margin-right: 10px;
  align-items: center;
  justify-content: center;
  min-width: 90px;
  ${({active}) =>
    active &&
    css`
      background-color: #fff;
    `};
`;

const Label = styled.Text<{active?: boolean}>`
  color: #fff;
  font-family: Gilroy-Bold;
  font-size: 14px;
  ${({active}) =>
    active &&
    css`
      color: #000;
    `};
`;

const Container = styled.View`
  flex-direction: row;
  align-items: center;
  height: 80px;
`;

export type FilterProps = {
  label: string;
  current: string;
  onSelectFilter: (filter: string) => void;
};

const Filter: FC<FilterProps> = ({label, current, onSelectFilter}) => {
  return (
    <Container>
      <TouchableWithoutFeedback onPress={() => onSelectFilter(label)}>
        <LabelWrapper active={current === label}>
          <Label active={current === label}>{label}</Label>
        </LabelWrapper>
      </TouchableWithoutFeedback>
    </Container>
  );
};

export type FiltersProps = {
  query: string;
  list: string[];
  current: string;
  onSelectFilter: (filter: string) => void;
};

const Filters: FC<FiltersProps> = ({query, list, current, onSelectFilter}) => {
  return (
    <Container>
      {query && (
        <FlatList
          data={list}
          horizontal={true}
          renderItem={({item}) => (
            <Filter
              label={item}
              current={current}
              onSelectFilter={onSelectFilter}
            />
          )}
        />
      )}
    </Container>
  );
};

export default Filters;
