import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button, Dialog, Icons, TextField } from "../UI";

const schema = z.object({
  handle: z
    .string()
    .trim()
    .min(1, "Enter your handle")
    // The `@` is how a handle is written everywhere, so it is the natural
    // thing to type; the daemon strips it. Rejecting it would be pedantry.
    .refine(
      (value) => value.replace(/^@/, "").includes("."),
      "A handle looks like you.bsky.social"
    ),
  password: z.string().min(1, "Enter an app password"),
});

export type SignInValues = z.infer<typeof schema>;

export type SignInModalProps = {
  isOpen: boolean;
  /** Rejected by the daemon — a wrong password, or a rate limit. */
  error?: string;
  submitting?: boolean;
  onClose: () => void;
  onSubmit: (values: SignInValues) => void;
};

/**
 * Sign in with an Atmosphere account.
 *
 * An **app password**, not the account password: atproto issues them for
 * exactly this, they can be revoked one at a time, and the daemon has no
 * browser to run an OAuth flow in. The form says so rather than leaving the
 * user to guess which password is wanted.
 */
const SignInModal = ({
  isOpen,
  error,
  submitting,
  onClose,
  onSubmit,
}: SignInModalProps) => {
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<SignInValues>({
    resolver: zodResolver(schema),
    defaultValues: { handle: "", password: "" },
  });

  // Reopening must not show the last attempt — least of all its password.
  useEffect(() => {
    if (isOpen) reset({ handle: "", password: "" });
  }, [isOpen, reset]);

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="Sign in with Atmosphere"
      icon={Icons.user}
      width={420}
      footer={
        <>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            form="sign-in"
            disabled={submitting}
            onClick={handleSubmit(onSubmit)}
          >
            {submitting ? "Signing in…" : "Sign in"}
          </Button>
        </>
      }
    >
      <form
        id="sign-in"
        className="flex flex-col gap-4 py-1"
        onSubmit={handleSubmit(onSubmit)}
      >
        <TextField
          label="HANDLE"
          placeholder="@atmosphere.handle"
          autoComplete="username"
          error={errors.handle?.message}
          {...register("handle")}
        />
        <TextField
          label="APP PASSWORD"
          type="password"
          placeholder="xxxx-xxxx-xxxx-xxxx"
          autoComplete="current-password"
          error={errors.password?.message}
          {...register("password")}
        />
        <p className="text-[11px] text-muted">
          An app password, not your account password — create one in your PDS
          settings. Signing in also enables scrobbling and station sync, which
          share this session.
        </p>
        <p className="border-t border-line pt-3 text-[11px] text-muted">
          No account yet? music-player is part of the Atmosphere.{" "}
          <a
            href="https://bsky.app"
            target="_blank"
            rel="noreferrer"
            className="text-accent hover:underline"
          >
            Create an Atmosphere account on Bluesky
          </a>{" "}
          to get started.
        </p>
        {error && (
          <p className="text-[11px] text-meter-high" role="alert">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
};

export default SignInModal;
