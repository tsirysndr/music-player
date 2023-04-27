import React, {FC} from 'react';
import {TouchableWithoutFeedback} from 'react-native';
import styled from '@emotion/native';

const Header = styled.View`
  margin: 0 20px;
  flex-direction: row;
  align-items: center;
  margin-bottom: 15px;
  justify-content: space-between;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 20px;
`;

export type SectionHeaderProps = {
  title: string;
  onSeeAll?: () => void;
  showSeeAll?: boolean;
};

const SeeAll = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
`;

const SeeAllContainer = styled.View`
  height: 30px;
  width: 60px;
  align-items: flex-end;
  justify-content: center;
`;

const SectionHeader: FC<SectionHeaderProps> = ({
  title,
  onSeeAll,
  showSeeAll,
}) => {
  return (
    <Header>
      <Title>{title}</Title>
      {showSeeAll && (
        <TouchableWithoutFeedback onPress={onSeeAll}>
          <SeeAllContainer>
            <SeeAll>See All</SeeAll>
          </SeeAllContainer>
        </TouchableWithoutFeedback>
      )}
    </Header>
  );
};

export default SectionHeader;
