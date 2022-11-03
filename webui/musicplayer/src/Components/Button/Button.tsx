import { FC } from "react";
import { Button as BaseButton, KIND } from "baseui/button";

export type ButtonProps = {
  onClick: () => void;
  children: string | JSX.Element;
  kind?: "primary" | "secondary";
  width?: string;
  height?: string;
  borderRadius?: string;
};

const Button: FC<ButtonProps> = ({
  children,
  kind,
  width,
  height,
  borderRadius,
}) => {
  return (
    <BaseButton
      kind={kind}
      overrides={{
        BaseButton: {
          style: {
            width,
            height,
            borderTopRadius: borderRadius,
            borderBottomRadius: borderRadius,
            borderLeftRadius: borderRadius,
            borderRightRadius: borderRadius,
            fontSize: "14px",
            fontFamily: "RockfordSansMedium",
          },
        },
      }}
    >
      {children}
    </BaseButton>
  );
};

Button.defaultProps = {
  kind: KIND.primary,
  width: "141px",
  height: "40px",
  borderRadius: "20px",
};

export default Button;
