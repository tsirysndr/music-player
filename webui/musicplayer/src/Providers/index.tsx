import { Toast } from "@heroui/react";
import { FC } from "react";
import SkinProvider from "./SkinProvider";

export type ProvidersProps = {
  children: React.ReactNode;
};

const Providers: FC<ProvidersProps> = ({ children }) => {
  return (
    <SkinProvider>
      {children}
      {/* The toast region has to be mounted for `toast(...)` calls anywhere in
          the tree to show up — device connect/disconnect notices go here. */}
      <Toast.Provider placement="bottom end" />
    </SkinProvider>
  );
};

export default Providers;
