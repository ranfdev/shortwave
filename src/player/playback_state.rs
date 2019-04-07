#[derive(Clone, PartialEq)]
pub enum PlaybackState {
    Playing,
    Stopped,
    Loading,
    Failure(String),
}
