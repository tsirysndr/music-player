import { keepPreviousData, useQueryClient } from "@tanstack/react-query";
import { FC, useMemo, useState } from "react";
import {
  useGetExtensionsQuery,
  useRescanExtensionsMutation,
  useSetExtensionEnabledMutation,
} from "../../Hooks/GraphQL";
import Extensions, { type StatusFilter } from "./Extensions";

const ExtensionsWithData: FC = () => {
  const [status, setStatus] = useState<StatusFilter>("all");
  const queryClient = useQueryClient();

  // No filter argument: searching extensions is the command palette's job now,
  // so this page always asks for the lot.
  const { data, isLoading, error } = useGetExtensionsQuery(
    {},
    { placeholderData: keepPreviousData }
  );

  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: ["GetExtensions"] });

  const setEnabled = useSetExtensionEnabledMutation({ onSettled: invalidate });
  const rescan = useRescanExtensionsMutation({ onSettled: invalidate });

  /**
   * The status chip filters here rather than on the daemon.
   *
   * It is a property of the stored flag, which the response already carries,
   * so a round trip would only add latency to a click that has all the
   * information it needs.
   */
  const extensions = useMemo(() => {
    const all = data?.extensions ?? [];
    if (status === "all") return all;
    return all.filter((extension) => extension.status === status);
  }, [data, status]);

  return (
    <Extensions
      extensions={extensions}
      loading={isLoading}
      error={error instanceof Error ? error.message : undefined}
      status={status}
      // The switch is held while its own mutation is in flight, so a double
      // click cannot queue two opposing writes.
      pending={setEnabled.isPending ? [setEnabled.variables?.id ?? ""] : []}
      onStatusFilter={setStatus}
      onToggle={(id, enabled) => setEnabled.mutate({ id, enabled })}
      onRescan={() => rescan.mutate({})}
    />
  );
};

export default ExtensionsWithData;
