use crate::{
    app::{ActiveBlock, App},
    event::Key,
};

pub fn handler(key: Key, app: &mut App) {
    match key {
        Key::Esc => {
            //app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::Library));
        }
        _ => (),
    }
}
