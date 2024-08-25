use extism::{convert::Json, *};

use crate::state::State;

host_fn!(pub connect_to_chromecast(app_data: State;) {
  Ok(())
});

host_fn!(pub reconnect_to_chromecast(app_data: State;) {
  Ok(())
});

host_fn!(pub send_command_to_chromecast(app_data: State; command: String) {
  Ok(())
});

host_fn!(pub chromecast_queue_load(app_data: State;) {
  Ok(())
});

host_fn!(pub load_track_to_chromecast(app_data: State;) {
  Ok(())
});

host_fn!(pub get_chromecast_current_playback(app_data: State;) {
  Ok(())
});

host_fn!(pub disconnect_from_chromecast(app_data: State;) {
  Ok(())
});
