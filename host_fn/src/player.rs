use extism::{convert::Json, *};

use crate::state::State;

host_fn!(pub load(app_data: State; url: String) {
  Ok(())
});

host_fn!(pub load_tracklist(app_data: State;) {
  Ok(())
});

host_fn!(pub preload(app_data: State;) {
  Ok(())
});

host_fn!(pub play(app_data: State;) {
  Ok(())
});

host_fn!(pub pause(app_data: State;) {
  Ok(())
});

host_fn!(pub stop(app_data: State;) {
  Ok(())
});

host_fn!(pub seek(app_data: State; position: u32) {
  Ok(())
});

host_fn!(pub play_track_at(app_data: State; index: u32) {
  Ok(())
});

host_fn!(pub clear(app_data: State;) {
  Ok(())
});

host_fn!(pub get_tracks(app_data: State;) {
  Ok(())
});

host_fn!(pub get_current_track(app_data: State;) {
  Ok(())
});

host_fn!(pub play_next(app_data: State;) {
  Ok(())
});

host_fn!(pub remove_track(app_data: State; index: u32) {
  Ok(())
});
