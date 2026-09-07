import type { IconProps as TablerIconProps } from "@tabler/icons-react";
import type { FC, RefAttributes } from "react";

/**
 * The props every icon takes, Tabler's shape.
 *
 * It lives in its own module so the generated `desktopIcons.tsx` can import it
 * without importing `icons.tsx`, which imports it back.
 */
export type IconProps = TablerIconProps;

/**
 * The type every icon in the set satisfies.
 *
 * Tabler's icons are `forwardRef` components, so they carry `RefAttributes`
 * that a plain `FC<IconProps>` does not accept; the generated ones are plain
 * function components. This union is what both fit through.
 */
export type IconComponent =
  | FC<IconProps>
  | FC<IconProps & RefAttributes<SVGSVGElement>>;
