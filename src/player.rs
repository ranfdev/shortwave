use gstreamer::prelude::*;
use gtk::prelude::*;
use rustio::{Client, Station};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::app::Action;
use crate::player::controller::{GtkController, MprisController};
use crate::song::Song;
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

pub struct Player {
    pub widget: gtk::Box,
    controller: Rc<Vec<Box<Controller>>>,

    backend: Arc<Mutex<PlayerBackend>>,
    song_listbox: Rc<RefCell<SongListBox>>,
}

impl Player {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/player.ui");
        let widget: gtk::Box = builder.get_object("player").unwrap();
        let backend = Arc::new(Mutex::new(PlayerBackend::new()));
        let song_listbox = Rc::new(RefCell::new(SongListBox::new()));
        widget.add(&song_listbox.borrow().widget);

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
            song_listbox,
        };

        player.setup_signals();
        player
    }

    pub fn set_station(&self, station: Station) {
        // discard old song, because it's not completely recorded
        self.song_listbox.borrow_mut().discard_current_song();

        self.song_listbox.borrow_mut().current_station = Some(station.clone());
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

                // We need to set it manually, because we don't receive a gst message when the playback stops
                for con in &*self.controller {
                    con.set_playback_state(&PlaybackState::Stopped);
                }
            }
            _ => (),
        };
    }

    pub fn shutdown(&self) {
        self.set_playback(PlaybackState::Stopped);
        self.song_listbox.borrow_mut().delete_everything();
    }

    fn parse_bus_message(message: &gstreamer::Message, controller: Rc<Vec<Box<Controller>>>, backend: Arc<Mutex<PlayerBackend>>, song_listbox: Rc<RefCell<SongListBox>>) {
        match message.view() {
            gstreamer::MessageView::Tag(tag) => {
                tag.get_tags().get::<gstreamer::tags::Title>().map(|t| {
                    let new_song = Song::new(t.get().unwrap());

                    // Check if song have changed
                    if song_listbox.borrow_mut().set_new_song(new_song.clone()) {
                        // set new song
                        debug!("New song: {:?}", new_song.clone().title);
                        for con in &*controller {
                            con.set_song_title(new_song.clone().title.as_ref());
                        }

                        debug!("Block the dataflow ...");
                        backend.lock().unwrap().block_dataflow();
                    }
                });
            }
            gstreamer::MessageView::StateChanged(sc) => {
                let playback_state = match sc.get_current() {
                    gstreamer::State::Playing => PlaybackState::Playing,
                    gstreamer::State::Paused => PlaybackState::Loading,
                    gstreamer::State::Ready => PlaybackState::Loading,
                    _ => PlaybackState::Stopped,
                };

                for con in &*controller {
                    con.set_playback_state(&playback_state);
                }
            }
            gstreamer::MessageView::Element(element) => {
                let structure = element.get_structure().unwrap();
                if structure.get_name() == "GstBinForwarded" {
                    let message: gstreamer::message::Message = structure.get("message").unwrap();
                    if let gstreamer::MessageView::Eos(_) = &message.view() {
                        debug!("muxsinkbin got EOS...");

                        if song_listbox.borrow().current_song.is_some() {
                            // Old song got saved correctly (cause we got the EOS message),
                            // so we can start with the new song now
                            let song = song_listbox.borrow_mut().current_song.clone().unwrap();
                            debug!("Cache song \"{}\" under \"{}\"", song.title, song.path);
                            backend.lock().unwrap().new_filesink_location(&song.path);
                        } else {
                            // Or just redirect the stream to /dev/null
                            backend.lock().unwrap().new_filesink_location("/dev/null");
                        }
                    }
                }
            }
            gstreamer::MessageView::Error(err) => {
                let msg = err.get_error().to_string();
                warn!("Gstreamer Error: {:?}", msg);
                for con in &*controller {
                    con.set_playback_state(&PlaybackState::Failure(msg.clone()));
                }
            }
            _ => (),
        };
    }

    fn setup_signals(&self) {
        // new backend (pipeline) bus messages
        let bus = self.backend.lock().unwrap().get_pipeline_bus();
        let controller = self.controller.clone();
        let backend = self.backend.clone();
        let song_listbox = self.song_listbox.clone();
        gtk::timeout_add(250, move || {
            while bus.have_pending() {
                bus.pop().map(|message| {
                    //debug!("new message {:?}", message);
                    Self::parse_bus_message(&message, controller.clone(), backend.clone(), song_listbox.clone());
                });
            }
            Continue(true)
        });
    }
}
