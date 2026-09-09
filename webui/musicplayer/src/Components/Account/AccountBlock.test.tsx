import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import AccountBlock from "./AccountBlock";

const signedIn = (overrides: Record<string, unknown> = {}) =>
  graphql.query("GetAccount", () =>
    HttpResponse.json({
      data: {
        account: {
          __typename: "Account",
          did: "did:plc:abc",
          handle: "tsiry.rocksky.app",
          displayName: "Tsiry",
          avatar: null,
          ...overrides,
        },
      },
    })
  );

const render = () => renderWithProviders(<AccountBlock />, { route: "/tracks" });

describe("AccountBlock", () => {
  it("offers to sign in when nobody is", async () => {
    render();
    expect(
      await screen.findByRole("button", { name: /Sign in/ })
    ).toBeInTheDocument();
  });

  it("shows the display name over the handle", async () => {
    server.use(signedIn());
    render();
    expect(await screen.findByText("Tsiry")).toBeInTheDocument();
    expect(screen.getByText("@tsiry.rocksky.app")).toBeInTheDocument();
  });

  /** Every account has a handle; a display name is optional. */
  it("falls back to the handle when there is no display name", async () => {
    server.use(signedIn({ displayName: null }));
    render();
    expect(await screen.findByText("tsiry.rocksky.app")).toBeInTheDocument();
  });

  /** Plenty of accounts have no avatar, so initials stand in. */
  it("draws initials when there is no avatar", async () => {
    server.use(signedIn());
    render();
    expect(await screen.findByText("T")).toBeInTheDocument();
  });

  describe("the sign-in form", () => {
    const open = async () => {
      const harness = render();
      await harness.user.click(
        await screen.findByRole("button", { name: /Sign in/ })
      );
      return harness;
    };

    it("asks for a handle and an app password", async () => {
      await open();
      expect(
        await screen.findByPlaceholderText("@atmosphere.handle")
      ).toBeInTheDocument();
      expect(await screen.findByLabelText("APP PASSWORD")).toBeInTheDocument();
    });

    /** A handle without a dot is not a handle. */
    it("refuses something that is not a handle", async () => {
      const { user } = await open();
      await user.type(await screen.findByLabelText("HANDLE"), "tsiry");
      await user.type(screen.getByLabelText("APP PASSWORD"), "hunter2");
      await user.click(screen.getByRole("button", { name: "Sign in" }));
      expect(
        await screen.findByText(/A handle looks like/)
      ).toBeInTheDocument();
    });

    /** The `@` is how a handle is written, so it must be accepted. */
    it("accepts a leading @", async () => {
      const seen = vi.fn();
      server.use(
        graphql.mutation("SignIn", ({ variables }) => {
          seen(variables);
          return HttpResponse.json({
            data: {
              signIn: {
                __typename: "Account",
                did: "did:plc:abc",
                handle: "tsiry.rocksky.app",
                displayName: "Tsiry",
                avatar: null,
              },
            },
          });
        })
      );

      const { user } = await open();
      await user.type(
        await screen.findByLabelText("HANDLE"),
        "@tsiry.rocksky.app"
      );
      await user.type(screen.getByLabelText("APP PASSWORD"), "hunter2");
      await user.click(screen.getByRole("button", { name: "Sign in" }));

      await waitFor(() => expect(seen).toHaveBeenCalled());
      expect(seen.mock.calls[0][0].handle).toBe("@tsiry.rocksky.app");
    });

    /** The daemon's own words: a wrong password and a rate limit differ. */
    it("shows why a sign-in was refused", async () => {
      server.use(
        graphql.mutation("SignIn", () =>
          HttpResponse.json({
            errors: [{ message: "that handle and app password did not match" }],
          })
        )
      );

      const { user } = await open();
      await user.type(
        await screen.findByLabelText("HANDLE"),
        "tsiry.rocksky.app"
      );
      await user.type(screen.getByLabelText("APP PASSWORD"), "wrong");
      await user.click(screen.getByRole("button", { name: "Sign in" }));

      expect(
        await screen.findByText(/did not match/)
      ).toBeInTheDocument();
    });
  });
});
