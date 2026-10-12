import { AppShell } from "../../Components/Layout";
import MediaBrowser from "./MediaBrowser";

export default function MediaPage() {
  return (
    <AppShell title="Media libraries">
      <MediaBrowser />
    </AppShell>
  );
}
