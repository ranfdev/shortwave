pub enum PlaybackState {
    Playing,
    Stopped,
    Loading,
    _Failure(String),
}
