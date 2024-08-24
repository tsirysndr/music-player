use extism::{convert::Json, *};

use crate::state::State;

host_fn!(pub connect_to_upnp_media_renderer(app_data: State;) {
  Ok(())
});

host_fn!(pub connect_to_upnp_media_server(app_data: State;) {
  Ok(())
});

host_fn!(pub browse_upnp_media_server(app_data: State;) {
  Ok(())
});

host_fn!(pub send_command_to_upnp_player(app_data: State; command: String) {
  Ok(())
});
