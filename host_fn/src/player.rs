use extism::{convert::Json, *};
use music_player_pdk::types::Track;
use music_player_playback::player::PlayerCommand;

use crate::state::State;

host_fn!(pub load(app_data: State; track: Json<Track>) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(
    PlayerCommand::LoadTracklist {
      tracks: vec![track.into_inner().into()],
  }
  ).unwrap();
  Ok(())
});

host_fn!(pub load_tracklist(app_data: State; tracks: Json<Vec<Track>>) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  let tracks = tracks.into_inner();
  let tracks = tracks.into_iter().map(Into::into).collect();
  cmd_tx.send(
    PlayerCommand::LoadTracklist {
      tracks: tracks,
  }
  ).unwrap();
  Ok(())
});

host_fn!(pub preload(app_data: State;) {
  todo!("Preload")
});

host_fn!(pub play(app_data: State;) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::Play)?;
  Ok(())
});

host_fn!(pub pause(app_data: State;) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::Pause)?;
  Ok(())
});

host_fn!(pub stop(app_data: State;) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::Stop)?;
  Ok(())
});

host_fn!(pub next(app_data: State;) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::Next)?;
  Ok(())
});

host_fn!(pub previous(app_data: State;) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::Previous)?;
  Ok(())
});

host_fn!(pub seek(app_data: State; position: u32) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::Seek(position))?;
  Ok(())
});

host_fn!(pub play_track_at(app_data: State; index: u32) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::PlayTrackAt(index as usize))?;
  Ok(())
});

host_fn!(pub clear(app_data: State;) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::Clear)?;
  Ok(())
});

host_fn!(pub get_current_track(app_data: State;) {
  todo!("get current track")
});

host_fn!(pub play_next(app_data: State; track: Json<Track>) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::PlayNext(track.into_inner().into()))?;
  Ok(())
});

host_fn!(pub remove_track(app_data: State; index: u32) {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  let cmd_tx = state.player_cmd_tx.lock().unwrap();
  cmd_tx.send(PlayerCommand::RemoveTrack(index as usize))?;
  Ok(())
});
