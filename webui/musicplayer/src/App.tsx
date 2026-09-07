import { useEffect, useState } from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import AppStateSync from "./Components/AppStateSync";
import AlbumDetailsPage from "./Containers/AlbumDetails";
import AlbumsPage from "./Containers/Albums";
import ArtistDetailsPage from "./Containers/ArtistDetails";
import ArtistsPage from "./Containers/Artists";
import ExtensionsPage from "./Containers/Extensions";
import FolderPage from "./Containers/Folder";
import LikedPage from "./Containers/Liked";
import PlaylistPage from "./Containers/Playlist";
import PlaylistsPage from "./Containers/Playlists";
import RadioPage from "./Containers/Radio";
import SearchPage from "./Containers/Search";
import ServersPage from "./Containers/Servers";
import TracksPage from "./Containers/Tracks";
import { resourceUriResolver } from "./ResourceUriResolver";

const hasNativeWrapper = !!import.meta.env.VITE_NATIVE_WRAPPER;

function App() {
  const [ready, setReady] = useState(!hasNativeWrapper);
  useEffect(() => {
    async function initializeForNativeWrapper() {
      if (!ready) {
        await resourceUriResolver.initializeForNativeWrapper();
        setReady(true);
      }
    }
    initializeForNativeWrapper();
  }, [ready]);
  if (!ready) return null;
  return (
    <BrowserRouter>
      <AppStateSync />
      <Routes>
        <Route path="/" element={<TracksPage />} />
        <Route path="/tracks" element={<TracksPage />} />
        <Route path="/artists" element={<ArtistsPage />} />
        <Route path="/albums" element={<AlbumsPage />} />
        <Route path="/albums/:id" element={<AlbumDetailsPage />} />
        <Route path="/artists/:id" element={<ArtistDetailsPage />} />
        <Route path="/search" element={<SearchPage />} />
        <Route path="/folders/:id" element={<FolderPage />} />
        <Route path="/playlists" element={<PlaylistsPage />} />
        <Route path="/playlists/:id" element={<PlaylistPage />} />
        <Route path="/liked" element={<LikedPage />} />
        <Route path="/radio" element={<RadioPage />} />
        <Route path="/extensions" element={<ExtensionsPage />} />
        <Route path="/servers" element={<ServersPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
