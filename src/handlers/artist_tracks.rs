use crate::{app::App, event::Key};

use super::tracks;

// The artist view's track list behaves exactly like the main track table.
pub fn handler(key: Key, app: &mut App) {
    tracks::handler(key, app);
}
