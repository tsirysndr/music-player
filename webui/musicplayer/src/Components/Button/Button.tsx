import { FC } from "react";
import { Button as BaseButton, KIND } from "baseui/button";

export type ButtonProps = {
  onClick: () => void;
  children: string | JSX.Element;
  kind?: "primary" | "secondary" | "tertiary";
  width?: string;
  height?: string;
  borderRadius?: string;
  disabled?: boolean;
};

const Button: FC<ButtonProps> = ({
  children,
  kind,
  width,
  height,
  borderRadius,
  onClick,
  disabled,
}) => {
  return (
    <BaseButton
      kind={kind}
      overrides={{
        BaseButton: {
          style: ({ $disabled }) => ({
            width,
            height,
            borderTopLeftRadius: borderRadius,
            borderTopRightRadius: borderRadius,
            borderBottomLeftRadius: borderRadius,
            borderBottomRightRadius: borderRadius,
            fontSize: "14px",
            fontFamily: "RockfordSansMedium",
            opacity: $disabled ? 0.7 : 1,
          }),
        },
      }}
      onClick={onClick}
      disabled={disabled}
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
  disabled: false,
};

export default Button;
