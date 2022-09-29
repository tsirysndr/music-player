#[derive(Debug)]
pub enum IoEvent {
    NextTrack,
    PreviousTrack,
}

pub struct Network {}

impl Network {
    pub async fn handle_network_event(&self, event: IoEvent) {}
}
