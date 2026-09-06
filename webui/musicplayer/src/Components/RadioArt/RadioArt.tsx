import styled from "@emotion/styled";
import { FC, useEffect, useState } from "react";
import { Radio } from "@styled-icons/feather";

const Logo = styled.img<{ size: number; radius: number }>`
  width: ${(props) => props.size}px;
  height: ${(props) => props.size}px;
  min-width: ${(props) => props.size}px;
  border-radius: ${(props) => props.radius}px;
  object-fit: cover;
  background-color: ${(props) => props.theme.colors.cover};
`;

const Placeholder = styled.div<{ size: number; radius: number }>`
  width: ${(props) => props.size}px;
  height: ${(props) => props.size}px;
  min-width: ${(props) => props.size}px;
  border-radius: ${(props) => props.radius}px;
  background-color: ${(props) => props.theme.colors.cover};
  display: flex;
  align-items: center;
  justify-content: center;
`;

export type RadioArtProps = {
  logo?: string;
  size?: number;
  radius?: number;
  className?: string;
};

/**
 * Station artwork with the shared fallback used everywhere a radio station is
 * rendered. Station logos come from third-party directories and are often
 * missing, expired or served as a format the browser refuses, so a failed load
 * falls back to the placeholder instead of leaving a broken image.
 */
const RadioArt: FC<RadioArtProps> = ({ logo, size, radius, className }) => {
  const [failed, setFailed] = useState(false);
  const boxSize = size!;
  const boxRadius = radius!;

  useEffect(() => setFailed(false), [logo]);

  if (!logo || failed) {
    return (
      <Placeholder size={boxSize} radius={boxRadius} className={className}>
        <Radio size={Math.round(boxSize * 0.45)} color="#ab28fc" />
      </Placeholder>
    );
  }

  return (
    <Logo
      src={logo}
      alt=""
      size={boxSize}
      radius={boxRadius}
      className={className}
      onError={() => setFailed(true)}
    />
  );
};

RadioArt.defaultProps = {
  size: 44,
  radius: 6,
};

export default RadioArt;
