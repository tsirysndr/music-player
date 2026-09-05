use crate::{app::App, event::Key};

use super::tracks;

// Album tracks behave exactly like the main track table: navigate, play on
// Enter, queue with the add-to-queue key.
pub fn handler(key: Key, app: &mut App) {
    tracks::handler(key, app);
}
