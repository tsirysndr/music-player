import { BrowserRouter, Routes, Route } from "react-router-dom";
import AlbumsPage from "./Containers/Albums";
import ArtistsPage from "./Containers/Artists";
import PlayQueuePage from "./Containers/PlayQueue";
import TracksPage from "./Containers/Tracks";
import AlbumDetailsPage from "./Containers/AlbumDetails";
import ArtistDetailsPage from "./Containers/ArtistDetails";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<TracksPage />} />
        <Route path="/tracks" element={<TracksPage />} />
        <Route path="/play-queue" element={<PlayQueuePage />} />
        <Route path="/artists" element={<ArtistsPage />} />
        <Route path="/albums" element={<AlbumsPage />} />
        <Route path="/albums/:id" element={<AlbumDetailsPage />} />
        <Route path="/artists/:id" element={<ArtistDetailsPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
