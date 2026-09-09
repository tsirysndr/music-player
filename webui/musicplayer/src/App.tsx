import { BrowserRouter, Route, Routes } from "react-router-dom";
import AppStateSync from "./Components/AppStateSync";
import AlbumDetailsPage from "./Containers/AlbumDetails";
import AlbumsPage from "./Containers/Albums";
import ArtistDetailsPage from "./Containers/ArtistDetails";
import ArtistsPage from "./Containers/Artists";
import ExtensionsPage from "./Containers/Extensions";
import GenreDetailPage from "./Components/Genres/GenreDetailWithData";
import GenresPage from "./Components/Genres";
import FolderPage from "./Containers/Folder";
import LikedPage from "./Containers/Liked";
import PlaylistPage from "./Containers/Playlist";
import PlaylistsPage from "./Containers/Playlists";
import RadioPage from "./Containers/Radio";
import SearchPage from "./Containers/Search";
import ServersPage from "./Containers/Servers";
import TracksPage from "./Containers/Tracks";
import { LibraryAnalysisProvider } from "./Components/UI";
import { useLibraryAnalyses } from "./Hooks/useLibraryAnalyses";

function App() {
  // Whether the connected library can answer for a key and a tempo. One query
  // for the whole app, rather than one per track list.
  const analyses = useLibraryAnalyses();

  return (
    <BrowserRouter>
      <AppStateSync />
      <LibraryAnalysisProvider value={analyses}>
        <Routes>
          <Route path="/" element={<TracksPage />} />
          <Route path="/tracks" element={<TracksPage />} />
          <Route path="/artists" element={<ArtistsPage />} />
          <Route path="/genres" element={<GenresPage />} />
          <Route path="/genres/:id" element={<GenreDetailPage />} />
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
      </LibraryAnalysisProvider>
    </BrowserRouter>
  );
}

export default App;
