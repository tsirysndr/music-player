import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionSourceAccount } from "../../GraphQL/SourceAccounts/Query";
import { renderWithProviders, screen, waitFor, within } from "../../test/render";
import { server } from "../../test/server";
import { SourceAccounts } from "./index";

const definition = {
  extensionId: "fixture", name: "Fixture source", enabled: true, sourceApiVersion: 1,
  configKeys: ["server", "password", "obsolete"],
};
const account = (overrides: Partial<ExtensionSourceAccount> = {}): ExtensionSourceAccount => ({
  id: "account-a", extensionId: "fixture", name: "Saved source", sourceApiVersion: 1,
  configRevision: "9007199254740993", identityBound: false,
  overriddenConfigKeys: ["server", "password", "obsolete"], ...overrides,
});

describe("extension source account management", () => {
  it("keeps defaults distinct from explicit empty values and connects only on request", async () => {
    let rows: ExtensionSourceAccount[] = [];
    const created = vi.fn();
    const connected = vi.fn();
    server.use(
      graphql.query("ExtensionSourceDefinitions", () => HttpResponse.json({
        data: { extensionSourceDefinitions: [definition] },
      })),
      graphql.query("SourceAccounts", () => HttpResponse.json({ data: { sourceAccounts: rows } })),
      graphql.mutation("CreateSourceAccount", ({ variables }) => {
        created(variables);
        const row = account({ id: "new-" + rows.length, name: String(variables.name) });
        rows = [...rows, row];
        return HttpResponse.json({ data: { createSourceAccount: row } });
      }),
      graphql.mutation("ConnectSourceAccount", ({ variables }) => {
        connected(variables);
        rows = rows.map((row) => row.id === variables.id ? { ...row, identityBound: true } : row);
        return HttpResponse.json({ data: { connectSourceAccount: rows.find((row) => row.id === variables.id) } });
      }),
    );
    const { user } = renderWithProviders(<SourceAccounts />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Add source account" })).toBeEnabled());
    await user.click(screen.getByRole("button", { name: "Add source account" }));
    await user.type(screen.getByLabelText("Account name"), "Defaults");
    await user.click(screen.getByRole("button", { name: "Save account" }));
    await screen.findByRole("button", { name: "Connect Defaults" });
    expect(created).toHaveBeenNthCalledWith(1, { extensionId: "fixture", name: "Defaults", config: [] });
    expect(connected).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Add source account" }));
    await user.type(screen.getByLabelText("Account name"), "Empty password");
    await user.selectOptions(screen.getByLabelText("Action for password"), "set");
    expect(screen.getByLabelText("Value for password")).toHaveValue("");
    await user.click(screen.getByRole("button", { name: "Save account" }));
    await user.click(await screen.findByRole("button", { name: "Connect Empty password" }));
    await screen.findByRole("link", { name: "Browse Empty password" });
    expect(created).toHaveBeenNthCalledWith(2, {
      extensionId: "fixture", name: "Empty password", config: [{ key: "password", value: "" }],
    });
    expect(connected).toHaveBeenCalledExactlyOnceWith({ id: "new-1" });
    expect(screen.queryByText(/Configuration revision|Identity bound/)).toBeNull();
  });

  it("sends exact CAS revision and Keep/Set empty/Remove; unavailable accounts remain editable and deletable", async () => {
    let rows = [account(), account({ id: "missing", extensionId: "uninstalled", name: "Offline account" })];
    const updated = vi.fn();
    const deleted = vi.fn();
    server.use(
      graphql.query("ExtensionSourceDefinitions", () => HttpResponse.json({
        data: { extensionSourceDefinitions: [{ ...definition, enabled: false }] },
      })),
      graphql.query("SourceAccounts", () => HttpResponse.json({ data: { sourceAccounts: rows } })),
      graphql.mutation("UpdateSourceAccount", ({ variables }) => {
        updated(variables);
        rows = rows.map((row) => row.id === variables.id ? { ...row, name: String(variables.name) } : row);
        return HttpResponse.json({ data: { updateSourceAccount: rows.find((row) => row.id === variables.id) } });
      }),
      graphql.mutation("DeleteSourceAccount", ({ variables }) => {
        deleted(variables);
        rows = rows.filter((row) => row.id !== variables.id);
        return HttpResponse.json({ data: { deleteSourceAccount: true } });
      }),
    );
    const { user } = renderWithProviders(<SourceAccounts />);
    await user.click(await screen.findByRole("button", { name: "Edit Saved source" }));
    expect(screen.getByRole("button", { name: "Connect Saved source" })).toBeDisabled();
    // The server key is left at Keep; no old credential value is requested.
    await user.selectOptions(screen.getByLabelText("Action for password"), "set");
    await user.selectOptions(screen.getByLabelText("Action for obsolete"), "remove");
    await user.click(screen.getByRole("button", { name: "Save account" }));
    await waitFor(() => expect(screen.queryByRole("form", { name: "Edit source account" })).toBeNull());
    expect(updated).toHaveBeenNthCalledWith(1, {
      id: "account-a", expectedRevision: "9007199254740993", name: "Saved source",
      changes: [{ key: "password", operation: "SET", value: "" }, { key: "obsolete", operation: "REMOVE" }],
    });
    await user.click(screen.getByRole("button", { name: "Edit Offline account" }));
    await user.clear(screen.getByLabelText("Account name"));
    await user.type(screen.getByLabelText("Account name"), "Renamed offline");
    await user.click(screen.getByRole("button", { name: "Save account" }));
    await user.click(await screen.findByRole("button", { name: "Delete Renamed offline" }));
    expect(updated).toHaveBeenNthCalledWith(2, {
      id: "missing", expectedRevision: "9007199254740993", name: "Renamed offline", changes: [],
    });
    await user.click(within(screen.getByRole("group", { name: "Confirm account deletion" }))
      .getByRole("button", { name: "Delete account" }));
    await waitFor(() => expect(screen.queryByRole("button", { name: "Edit Renamed offline" })).toBeNull());
    expect(deleted).toHaveBeenCalledExactlyOnceWith({ id: "missing" });
  });
});
