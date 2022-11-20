import { BrowserRouter, Routes, Route } from "react-router-dom";
import AlbumsPage from "./Containers/Albums";
import ArtistsPage from "./Containers/Artists";
import TracksPage from "./Containers/Tracks";
import AlbumDetailsPage from "./Containers/AlbumDetails";
import ArtistDetailsPage from "./Containers/ArtistDetails";
import SearchPage from "./Containers/Search";
import PlaylistPage from "./Containers/Playlist";
import FolderPage from "./Containers/Folder";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<TracksPage />} />
        <Route path="/tracks" element={<TracksPage />} />
        <Route path="/artists" element={<ArtistsPage />} />
        <Route path="/albums" element={<AlbumsPage />} />
        <Route path="/albums/:id" element={<AlbumDetailsPage />} />
        <Route path="/artists/:id" element={<ArtistDetailsPage />} />
        <Route path="/search" element={<SearchPage />} />
        <Route path="/folders/:id" element={<FolderPage />} />
        <Route path="/playlists/:id" element={<PlaylistPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
