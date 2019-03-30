use gio::prelude::*;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use rustio::{Client, Station};

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app::Action;
use crate::config;
use crate::player::controller::{GtkController, MprisController};
use crate::player::gstreamer_backend::GstreamerMessage;
use crate::widgets::song_listbox::SongListBox;

////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                        //
//  A small overview of the player/gstreamer program structure  :)                        //
//                                                                                        //
//   ----------------------    -----------------    ---------------                       //
//  | ChromecastController |  | MprisController |  | GtkController |                      //
//   ----------------------    -----------------    ---------------                       //
//            |                        |                   |                              //
//            ----------------------------------------------                              //
//                                     |                                                  //
//                              ------------                          --------------      //
//                             | Controller |                        | AudioBackend |     //
//                              ------------                          --------------      //
//                                     |      -------------------           |             //
//                                     |     | Gstreamer Backend |----------|             //
//	                                   |      -------------------           |             //
//                                     |        |                     ---------------     //
//                                    -----------                    | ExportBackend |    //
//                                   |  Player   |                    ---------------     //
//                                    -----------                                         //
//                                                                                        //
////////////////////////////////////////////////////////////////////////////////////////////

mod controller;
pub mod gstreamer_backend;
mod playback_state;

pub use controller::Controller;
pub use gstreamer_backend::PlayerBackend;
pub use playback_state::PlaybackState;

use crate::model::SongModel;

pub struct Player {
    pub widget: gtk::Box,
    controller: Rc<Vec<Box<Controller>>>,

    backend: Arc<Mutex<PlayerBackend>>,
    song_model: Rc<RefCell<SongModel>>,
    song_listbox: SongListBox,
}

impl Player {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/player.ui");
        let widget: gtk::Box = builder.get_object("player").unwrap();

        let song_model = Rc::new(RefCell::new(SongModel::new(5)));
        let song_listbox = SongListBox::new(sender.clone());
        song_listbox.bind_model(&song_model.borrow());
        widget.add(&song_listbox.widget);

        let (gst_sender, gst_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let backend = Arc::new(Mutex::new(PlayerBackend::new(gst_sender)));

        let mut controller: Vec<Box<Controller>> = Vec::new();

        // Gtk Controller
        let gtk_controller = GtkController::new(sender.clone());
        let controller_box: gtk::Box = builder.get_object("controller_box").unwrap();
        controller_box.add(&gtk_controller.widget);
        controller.push(Box::new(gtk_controller));

        // Mpris Controller
        let mpris_controller = MprisController::new(sender.clone());
        controller.push(Box::new(mpris_controller));

        let controller: Rc<Vec<Box<Controller>>> = Rc::new(controller);

        let player = Self {
            widget,
            controller,
            backend,
            song_model,
            song_listbox,
        };

        player.setup_signals(gst_receiver);
        player
    }

    pub fn set_station(&self, station: Station) {
        // TODO: discard old song (because it's not completely recorded),
        // stop recording and stop playback
        let song = self.backend.lock().unwrap().stop_recording().unwrap();
        self.song_model.borrow_mut().remove_song(&song);
        self.set_playback(PlaybackState::Stopped);

        for con in &*self.controller {
            con.set_station(station.clone());
        }

        let backend = self.backend.clone();
        thread::spawn(move || {
            let mut client = Client::new("http://www.radio-browser.info");
            let station_url = client.get_playable_station_url(station).unwrap();
            debug!("new source uri to record: {}", station_url);
            backend.lock().unwrap().new_source_uri(&station_url);
        });
    }

    pub fn set_playback(&self, playback: PlaybackState) {
        match playback {
            PlaybackState::Playing => {
                let _ = self.backend.lock().unwrap().set_state(gstreamer::State::Playing);
            }
            PlaybackState::Stopped => {
                let _ = self.backend.lock().unwrap().set_state(gstreamer::State::Null);
            }
            _ => (),
        }
    }

    pub fn shutdown(&self) {
        self.set_playback(PlaybackState::Stopped);

        // Clear song model and remove all saved songs
        self.song_model.borrow_mut().clear();
        fs::remove_dir_all(Self::get_song_path("".to_string())).expect("Could not remove recording folder");
    }

    fn setup_signals(&self, receiver: Receiver<GstreamerMessage>) {
        // Wait for new messages from the Gstreamer backend
        let controller = self.controller.clone();
        let song_model = self.song_model.clone();
        let backend = self.backend.clone();
        receiver.attach(None, move |message| Self::process_gst_message(message, controller.clone(), song_model.clone(), backend.clone()));

        // Show song listbox if a song gets added
        let listbox = self.song_listbox.widget.clone();
        self.song_model.borrow().model.connect_items_changed(move |_, _, _, added| {
            if added == 1 {
                listbox.set_visible(true);
            }
        });
    }

    fn process_gst_message(message: GstreamerMessage, controller: Rc<Vec<Box<Controller>>>, song_model: Rc<RefCell<SongModel>>, backend: Arc<Mutex<PlayerBackend>>) -> glib::Continue {
        match message {
            GstreamerMessage::SongTitleChanged(title) => {
                debug!("Song title has changed: \"{}\"", title);

                for con in &*controller {
                    con.set_song_title(&title);
                }

                // Song have changed -> stop recording
                if backend.lock().unwrap().is_recording() {
                    let song = backend.lock().unwrap().stop_recording().unwrap();
                    song_model.borrow_mut().add_song(song);
                } else {
                    // Get current/new song title
                    let title = backend.lock().unwrap().get_current_song_title();

                    // Nothing needs to be stopped, so we can start directly recording.
                    backend.lock().unwrap().start_recording(Self::get_song_path(title));
                }
            }
            GstreamerMessage::RecordingStopped => {
                // Recording successfully stopped.
                debug!("Recording stopped.");

                // Get current/new song title
                let title = backend.lock().unwrap().get_current_song_title();

                // Start recording new song
                if title != "" {
                    backend.lock().unwrap().start_recording(Self::get_song_path(title));
                }
            }
            GstreamerMessage::PlaybackStateChanged(state) => {
                for con in &*controller {
                    con.set_playback_state(&state);
                }
            }
        }
        glib::Continue(true)
    }

    fn get_song_path(mut title: String) -> PathBuf {
        // remove special chars from title
        // if anybody knows a better way to do this, feel free to open a MR on GitLab :)
        title = title.replace("/", "");
        title = title.replace("\\", "");
        title = title.replace(":", "");
        title = title.replace("<", "");
        title = title.replace(">", "");
        title = title.replace("\"", "");
        title = title.replace("|", "");
        title = title.replace("?", "");
        title = title.replace("*", "");

        let mut path = glib::get_user_cache_dir().unwrap();
        path.push(config::NAME);
        path.push("recording");

        // Make sure that the path exists
        fs::create_dir_all(path.clone()).expect("Could not create path for recording");

        if title != "" {
            path.push(title);
            path.set_extension("ogg");
        }
        path
    }
}
