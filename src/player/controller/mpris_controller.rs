use glib::Sender;
use mpris_player::{Metadata, MprisPlayer, OrgMprisMediaPlayer2Player, PlaybackStatus};
use rustio::Station;

use std::cell::Cell;
use std::sync::Arc;

use crate::app::Action;
use crate::player::Controller;
use crate::player::PlaybackState;

pub struct MprisController {
    sender: Sender<Action>,
    mpris: Arc<MprisPlayer>,

    song_title: Cell<Option<String>>,
    station: Cell<Option<Station>>,
}

impl MprisController {
    pub fn new(sender: Sender<Action>) -> Self {
        let mpris = MprisPlayer::new("Shortwave".to_string(), "Shortwave".to_string(), "de.haeckerfelix.Shortwave".to_string());
        mpris.set_can_raise(true);
        mpris.set_can_play(false);
        mpris.set_can_seek(false);
        mpris.set_can_set_fullscreen(false);
        mpris.set_can_pause(true);

        let controller = Self {
            sender,
            mpris,
            song_title: Cell::new(None),
            station: Cell::new(None),
        };

        controller.connect_signals();
        controller
    }

    fn update_metadata(&self) {
        let mut metadata = Metadata::new();

        let station = self.station.take();
        let song_title = self.song_title.take();

        station.clone().map(|station| {
            metadata.art_url = Some(station.favicon);
            metadata.artist = Some(vec![station.name]);
        });
        song_title.clone().map(|song_title| {
            metadata.title = Some(song_title);
        });

        self.station.set(station);
        self.song_title.set(song_title);

        self.mpris.set_metadata(metadata);
    }

    fn connect_signals(&self) {
        // mpris raise
        let sender = self.sender.clone();
        self.mpris.connect_raise(move || {
            sender.send(Action::ViewRaise).unwrap();
        });

        // mpris play / pause
        let sender = self.sender.clone();
        let mpris = self.mpris.clone();
        self.mpris.connect_play_pause(move || {
            match mpris.get_playback_status().unwrap().as_ref() {
                "Paused" => sender.send(Action::PlaybackStart).unwrap(),
                "Stopped" => sender.send(Action::PlaybackStart).unwrap(),
                _ => sender.send(Action::PlaybackStop).unwrap(),
            };
        });

        // mpris play
        let sender = self.sender.clone();
        self.mpris.connect_play(move || {
            sender.send(Action::PlaybackStart).unwrap();
        });

        // mpris stop
        let sender = self.sender.clone();
        self.mpris.connect_stop(move || {
            sender.send(Action::PlaybackStop).unwrap();
        });

        // mpris pause
        let sender = self.sender.clone();
        self.mpris.connect_pause(move || {
            sender.send(Action::PlaybackStop).unwrap();
        });
    }
}

impl Controller for MprisController {
    fn set_station(&self, station: Station) {
        self.station.set(Some(station));
        self.update_metadata();
    }

    fn set_playback_state(&self, playback_state: &PlaybackState) {
        self.mpris.set_can_play(true);

        match playback_state {
            PlaybackState::Playing => self.mpris.set_playback_status(PlaybackStatus::Playing),
            _ => self.mpris.set_playback_status(PlaybackStatus::Stopped),
        };
    }

    fn set_song_title(&self, title: &str) {
        self.song_title.set(Some(title.to_string()));
        self.update_metadata();
    }
}
