use extism::{convert::Json, host_fn};
use music_player_pdk::types::Tracklist;

use crate::state::State;

host_fn!(pub get_current_tracklist(app_data: State;) -> Result<Json<Tracklist>, Error> {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let tracklist = state.tracklist.lock().unwrap();
  let (previous_tracks, next_tracks) = tracklist.tracks();
  Ok(Json(Tracklist {
    previous_tracks: previous_tracks.into_iter().map(Into::into).collect(),
    next_tracks: next_tracks.into_iter().map(Into::into).collect(),
  }))
});
