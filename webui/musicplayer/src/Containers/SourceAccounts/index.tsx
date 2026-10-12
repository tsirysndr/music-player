import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
import { AppShell } from "../../Components/Layout";
import { Button } from "../../Components/UI";
import {
  GET_EXTENSION_SOURCE_DEFINITIONS, GET_SOURCE_ACCOUNTS,
  type ExtensionSourceAccount, type ExtensionSourceDefinition,
  type ExtensionSourceDefinitionsQuery, type SourceAccountsQuery,
} from "../../GraphQL/SourceAccounts/Query";
import {
  CONNECT_SOURCE_ACCOUNT, CREATE_SOURCE_ACCOUNT, DELETE_SOURCE_ACCOUNT, UPDATE_SOURCE_ACCOUNT,
  type ConnectSourceAccountMutation, type CreateSourceAccountMutation,
  type DeleteSourceAccountMutation, type SourceConfigChangeInput, type UpdateSourceAccountMutation,
} from "../../GraphQL/SourceAccounts/Mutation";
import { errorMessage, sourceRequest, useSourceOperation } from "./requests";

const fieldClass = "w-full rounded-control border border-line bg-panel p-2 text-sm text-fg";
type ConfigEdit = { mode: "keep" | "set" | "remove"; value: string };

export default function SourceAccountsPage() {
  return <AppShell title="Extension source accounts"><SourceAccounts /></AppShell>;
}

export function SourceAccounts() {
  const [editor, setEditor] = useState<ExtensionSourceAccount | "new" | null>(null);
  const [deleting, setDeleting] = useState<ExtensionSourceAccount | null>(null);
  const operation = useSourceOperation();
  const data = useQuery({
    queryKey: ["ExtensionSourceAccounts"],
    queryFn: async ({ signal }) => {
      const [accounts, definitions] = await Promise.all([
        sourceRequest<SourceAccountsQuery>(GET_SOURCE_ACCOUNTS, {}, signal),
        sourceRequest<ExtensionSourceDefinitionsQuery>(GET_EXTENSION_SOURCE_DEFINITIONS, {}, signal),
      ]);
      return { accounts: accounts.sourceAccounts, definitions: definitions.extensionSourceDefinitions };
    },
    retry: false,
  });
  const refresh = () => { void data.refetch(); };
  const edit = (account: ExtensionSourceAccount | "new") => {
    operation.cancel();
    setDeleting(null);
    setEditor(account);
  };
  const connect = (account: ExtensionSourceAccount) => {
    void operation.run(async (attempt) => {
      const result = await attempt.submit(() => sourceRequest<ConnectSourceAccountMutation>(
        CONNECT_SOURCE_ACCOUNT, { id: account.id }, attempt.signal, true,
      ));
      if (result.connectSourceAccount.id !== account.id) throw new Error("Unexpected account response");
      return result.connectSourceAccount;
    }, () => { refresh(); }, "Account connected. Playback has not been changed.");
  };
  const remove = (account: ExtensionSourceAccount) => {
    void operation.run(async (attempt) => {
      const result = await attempt.submit(() => sourceRequest<DeleteSourceAccountMutation>(
        DELETE_SOURCE_ACCOUNT, { id: account.id }, attempt.signal, true,
      ));
      if (!result.deleteSourceAccount) throw new Error("The account was not deleted");
    }, () => { setDeleting(null); refresh(); }, "Account deleted.");
  };

  return <section aria-label="Extension source accounts" className="space-y-4 py-4">
    <div className="flex flex-wrap gap-2">
      <Link to="/extensions" className="text-sm text-accent underline">Installed extensions</Link>
      <Button variant="outline" onClick={() => { operation.cancel(); refresh(); }}>Refresh accounts</Button>
      <Button disabled={!data.data?.definitions.some((definition) => definition.enabled)}
        onClick={() => edit("new")}>Add source account</Button>
    </div>
    <p className="text-sm text-muted">Save separate accounts for each source. Connect explicitly after saving configuration.</p>
    {operation.busy && <Button variant="outline" onClick={operation.cancel}>Stop waiting</Button>}
    {operation.message && <p role="status">{operation.message}</p>}
    {operation.error && <p role="alert">{operation.error}</p>}
    {data.isPending && <p role="status">Loading source accounts…</p>}
    {data.isError && <div className="space-y-2">
      <p role="alert">{errorMessage(data.error)}</p>
      <Button variant="outline" onClick={refresh}>Retry accounts</Button>
    </div>}
    {editor && data.data && <AccountForm key={editor === "new" ? "new" : editor.id + ":" + editor.configRevision}
      account={editor === "new" ? undefined : editor} definitions={data.data.definitions}
      onSaved={() => { setEditor(null); refresh(); }} onClose={() => setEditor(null)} />}
    {deleting && <div role="group" aria-label="Confirm account deletion" className="space-y-2 rounded-control border border-line p-3">
      <p>Delete {deleting.name}? Saved queue references to this account will no longer resolve.</p>
      <Button variant="danger" disabled={operation.busy} onClick={() => remove(deleting)}>Delete account</Button>
      <Button variant="ghost" onClick={() => { operation.cancel(); setDeleting(null); }}>Keep account</Button>
    </div>}
    {data.data?.accounts.length === 0 && <p>No source accounts saved.</p>}
    <ul className="space-y-3">
      {data.data?.accounts.map((account) => {
        const definition = data.data?.definitions.find((item) => item.extensionId === account.extensionId);
        const enabled = definition?.enabled === true;
        return <li key={account.id} className="space-y-2 rounded-skin bg-panel p-4">
          <h2 className="break-words text-base font-semibold">{account.name}</h2>
          <p className="text-xs text-muted">{definition?.name ?? account.extensionId} · v{account.sourceApiVersion}
            {enabled ? "" : " · Disabled or unavailable"} · {account.identityBound ? "Connected" : "Connection required"}</p>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={() => edit(account)}>Edit {account.name}</Button>
            <Button disabled={!enabled || operation.busy || editor !== null} onClick={() => connect(account)}>
              Connect {account.name}
            </Button>
            {enabled && account.identityBound && <Link className="self-center text-sm text-accent underline"
              to={"/source-accounts/" + encodeURIComponent(account.id)} onClick={operation.cancel}>Browse {account.name}</Link>}
            <Button variant="ghost" onClick={() => {
              operation.cancel(); setEditor(null); setDeleting(account);
            }}>Delete {account.name}</Button>
          </div>
        </li>;
      })}
    </ul>
  </section>;
}

function AccountForm({ account, definitions, onSaved, onClose }: {
  account?: ExtensionSourceAccount; definitions: ExtensionSourceDefinition[];
  onSaved: () => void; onClose: () => void;
}) {
  const [extensionId, setExtensionId] = useState(account?.extensionId ?? definitions.find((item) => item.enabled)?.extensionId ?? "");
  const [name, setName] = useState(account?.name ?? "");
  const [edits, setEdits] = useState<Record<string, ConfigEdit>>({});
  const operation = useSourceOperation();
  const definition = definitions.find((item) => item.extensionId === extensionId);
  const keys = [...new Set([...(definition?.configKeys ?? []), ...(account?.overriddenConfigKeys ?? [])])];
  const save = () => {
    void operation.run<CreateSourceAccountMutation | UpdateSourceAccountMutation>(async (attempt) => {
      if (account) {
        const changes = keys.flatMap<SourceConfigChangeInput>((key) => {
          const edit = edits[key];
          if (!edit || edit.mode === "keep") return [];
          return edit.mode === "set" ? [{ key, operation: "SET" as const, value: edit.value }]
            : [{ key, operation: "REMOVE" as const }];
        });
        return attempt.submit(() => sourceRequest<UpdateSourceAccountMutation>(UPDATE_SOURCE_ACCOUNT, {
          id: account.id, expectedRevision: account.configRevision, name, changes,
        }, attempt.signal, true));
      }
      const config = keys.filter((key) => edits[key]?.mode === "set")
        .map((key) => ({ key, value: edits[key].value }));
      return attempt.submit(() => sourceRequest<CreateSourceAccountMutation>(CREATE_SOURCE_ACCOUNT, {
        extensionId, name, config,
      }, attempt.signal, true));
    }, onSaved);
  };
  return <form aria-label={account ? "Edit source account" : "Create source account"}
    className="space-y-3 rounded-control border border-line p-4"
    onSubmit={(event) => { event.preventDefault(); save(); }}>
    <h2 className="font-semibold">{account ? "Edit source account" : "Create source account"}</h2>
    {!account && <label className="block text-sm">Source extension
      <select className={fieldClass} value={extensionId} disabled={operation.busy} onChange={(event) => {
        setExtensionId(event.target.value); setEdits({});
      }}>
        {definitions.filter((item) => item.enabled).map((item) => <option key={item.extensionId} value={item.extensionId}>{item.name}</option>)}
      </select>
    </label>}
    <label className="block text-sm">Account name
      <input required className={fieldClass} value={name} disabled={operation.busy} onChange={(event) => setName(event.target.value)} />
    </label>
    <p className="text-xs text-muted">Saved values are not displayed. Set value accepts an empty string. Use extension default removes an override.</p>
    {keys.map((key) => {
      const edit = edits[key] ?? { mode: "keep", value: "" };
      return <fieldset key={key} disabled={operation.busy} className="space-y-2">
        <legend className="break-words text-sm">{key}{account?.overriddenConfigKeys.includes(key) ? " (override saved)" : ""}</legend>
        <select aria-label={"Action for " + key} className={fieldClass} value={edit.mode} onChange={(event) => {
          setEdits((old) => ({ ...old, [key]: { mode: event.target.value as ConfigEdit["mode"], value: "" } }));
        }}>
          <option value="keep">{account ? "Keep saved value" : "Use extension default"}</option>
          <option value="set" disabled={!definition?.configKeys.includes(key)}>Set value (including empty)</option>
          {account && <option value="remove">Use extension default</option>}
        </select>
        {edit.mode === "set" && <input type="password" autoComplete="new-password" aria-label={"Value for " + key}
          className={fieldClass} value={edit.value} onChange={(event) => setEdits((old) => ({
            ...old, [key]: { mode: "set", value: event.target.value },
          }))} />}
      </fieldset>;
    })}
    {operation.error && <p role="alert">{operation.error}</p>}
    {operation.message && <p role="status">{operation.message}</p>}
    <div className="flex flex-wrap gap-2">
      <Button type="submit" disabled={operation.busy || !name.trim() || !extensionId}>Save account</Button>
      {operation.busy && <Button variant="outline" onClick={operation.cancel}>Stop waiting for save</Button>}
      <Button variant="ghost" onClick={() => { operation.cancel(); onClose(); }}>Close editor</Button>
    </div>
    <p className="text-xs text-muted">Saving does not connect. Stopping the wait or closing this editor cannot undo a save already sent.</p>
  </form>;
}
