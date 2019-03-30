#[derive(Clone)]
pub enum PlaybackState {
    Playing,
    Stopped,
    Loading,
    Failure(String),
}
