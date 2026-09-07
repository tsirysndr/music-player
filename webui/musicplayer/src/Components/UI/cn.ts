import clsx, { type ClassValue } from "clsx";

/**
 * Class-name join. Deliberately not a tailwind-merge: the components here own
 * their own classes and callers add to them rather than overriding, so the
 * last-wins conflict resolution would cost more than it buys.
 */
export const cn = (...classes: ClassValue[]) => clsx(classes);

export default cn;
