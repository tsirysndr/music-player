import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  useGetAccountQuery,
  useSignInMutation,
  useSignOutMutation,
} from "../../Hooks/GraphQL";
import { Button, ContextMenu, ContextMenuItem, Icons, cn } from "../UI";
import SignInModal, { type SignInValues } from "./SignInModal";

/** The initials an avatar-less account is drawn as. */
const initialsOf = (label: string) =>
  label
    .replace(/^@/, "")
    .split(/[\s.]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("") || "?";

/**
 * The signed-in account, at the foot of the sidebar.
 *
 * One account for the whole daemon — the same session scrobbling and station
 * sync use — so this is the only place to sign in, and signing out here stops
 * those too.
 */
const AccountBlock = ({ collapsed }: { collapsed?: boolean }) => {
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [error, setError] = useState<string>();

  const { data } = useGetAccountQuery();
  const account = data?.account;
  const signIn = useSignInMutation();
  const signOut = useSignOutMutation();

  const refresh = () =>
    queryClient.invalidateQueries({
      predicate: (query) => query.queryKey[0] === "GetAccount",
    });

  if (!account) {
    return (
      <>
        <Button
          variant="outline"
          className={cn("w-full justify-center", collapsed && "px-0")}
          onClick={() => {
            setError(undefined);
            setOpen(true);
          }}
        >
          <Icons.user size={14} />
          {!collapsed && "Sign in"}
        </Button>
        <SignInModal
          isOpen={open}
          error={error}
          submitting={signIn.isPending}
          onClose={() => setOpen(false)}
          onSubmit={async (values: SignInValues) => {
            setError(undefined);
            try {
              await signIn.mutateAsync({
                handle: values.handle,
                password: values.password,
              });
              await refresh();
              setOpen(false);
            } catch (cause) {
              setError(
                cause instanceof Error ? cause.message : "Could not sign in"
              );
            }
          }}
        />
      </>
    );
  }

  const label = account.displayName?.trim() || account.handle;

  return (
    <ContextMenu
      placement="top start"
      trigger={
        <button
          type="button"
          className="flex w-full items-center gap-[10px] rounded-control px-2 py-[6px] text-left hover:bg-hover"
          aria-label={`Signed in as ${label}`}
        >
          {account.avatar ? (
            <img
              src={account.avatar}
              alt=""
              className="size-7 shrink-0 rounded-full object-cover"
            />
          ) : (
            <span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-panel text-[10px] font-semibold text-dim">
              {initialsOf(label)}
            </span>
          )}
          {!collapsed && (
            <span className="flex min-w-0 flex-1 flex-col">
              <span className="truncate text-[12px] font-semibold text-fg">
                {label}
              </span>
              {/* The handle is the identity; muted because the name above is
                  what the eye should land on. */}
              <span className="truncate text-[11px] text-muted">
                @{account.handle}
              </span>
            </span>
          )}
        </button>
      }
    >
      <ContextMenuItem
        icon={Icons.logout}
        label="Sign out"
        danger
        onClick={async () => {
          await signOut.mutateAsync({});
          await refresh();
        }}
      />
    </ContextMenu>
  );
};

export default AccountBlock;
