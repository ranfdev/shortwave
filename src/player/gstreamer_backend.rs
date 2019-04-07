use glib::Sender;
use gstreamer::prelude::*;
use gstreamer::{Bin, Element, ElementFactory, GhostPad, Pad, PadProbeId, Pipeline, State};

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::player::playback_state::PlaybackState;
use crate::song::Song;

//////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                      //
//  # Gstreamer Pipeline                                                                                //
//                                            -----      --------       -------------                   //
//                                           |     | -> | queue [1] -> | recorderbin |                  //
//    --------------      --------------     |     |     --------       -------------                   //
//   | uridecodebin | -> | audioconvert | -> | tee |                                                    //
//    --------------      --------------     |     |     -------      --------      ---------------     //
//                                           |     | -> | queue | -> | volume | -> | autoaudiosink |    //
//                                            -----      -------      --------      ---------------     //
//                                                                                                      //
//                                                                                                      //
//                                                                                                      //
//  We use the the file_srcpad[1] to block the dataflow, so we can change the recorderbin.              //
//  The dataflow gets blocked when the song changes.                                                    //
//                                                                                                      //
//////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum GstreamerMessage {
    SongTitleChanged(String),
    PlaybackStateChanged(PlaybackState),
    RecordingStopped,
}

#[allow(dead_code)]
pub struct GstreamerBackend {
    pipeline: Pipeline,

    uridecodebin: Element,
    audioconvert: Element,
    tee: Element,

    audio_queue: Element,
    volume: Element,
    autoaudiosink: Element,

    file_queue: Element,
    recorderbin: Arc<Mutex<Option<RecorderBin>>>,
    file_srcpad: Pad,
    file_blockprobe_id: Option<PadProbeId>,

    current_title: Arc<Mutex<String>>,
    sender: Sender<GstreamerMessage>,
}

impl GstreamerBackend {
    pub fn new(sender: Sender<GstreamerMessage>) -> Self {
        // create gstreamer pipeline
        let pipeline = Pipeline::new("recorder_pipeline");

        // create pipeline elements
        let uridecodebin = ElementFactory::make("uridecodebin", "uridecodebin").unwrap();
        let audioconvert = ElementFactory::make("audioconvert", "audioconvert").unwrap();
        let tee = ElementFactory::make("tee", "tee").unwrap();
        let audio_queue = ElementFactory::make("queue", "audio_queue").unwrap();
        let volume = ElementFactory::make("volume", "volume").unwrap();
        let autoaudiosink = ElementFactory::make("autoaudiosink", "autoaudiosink").unwrap();
        let file_queue = ElementFactory::make("queue", "file_queue").unwrap();
        let file_srcpad = file_queue.get_static_pad("src").unwrap();

        // link pipeline elements
        pipeline.add_many(&[&uridecodebin, &audioconvert, &tee, &audio_queue, &volume, &autoaudiosink, &file_queue]).unwrap();
        Element::link_many(&[&audioconvert, &tee]).unwrap();
        let tee_tempmlate = tee.get_pad_template("src_%u").unwrap();

        // link tee -> queue
        let tee_file_srcpad = tee.request_pad(&tee_tempmlate, None, None).unwrap();
        let _ = tee_file_srcpad.link(&file_queue.get_static_pad("sink").unwrap());

        // link tee -> queue -> volume -> autoaudiosink
        let tee_audio_srcpad = tee.request_pad(&tee_tempmlate, None, None).unwrap();
        let _ = tee_audio_srcpad.link(&audio_queue.get_static_pad("sink").unwrap());
        let _ = audio_queue.link(&volume);
        let _ = volume.link(&autoaudiosink);

        // dynamically link uridecodebin element with audioconvert element
        let convert = audioconvert.clone();
        uridecodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = convert.get_static_pad("sink").expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                return; // We are already linked. Ignoring.
            }

            let new_pad_caps = src_pad.get_current_caps().expect("Failed to get caps of new pad.");
            let new_pad_struct = new_pad_caps.get_structure(0).expect("Failed to get first structure of caps.");
            let new_pad_type = new_pad_struct.get_name();

            if new_pad_type.starts_with("audio/x-raw") {
                // check if new_pad is audio
                let _ = src_pad.link(&sink_pad);
                return;
            }
        });

        // Current song title. We need this variable to check if the title have changed.
        let current_title = Arc::new(Mutex::new(String::new()));

        // listen for new pipeline / bus messages
        let ct = current_title.clone();
        let bus = pipeline.get_bus().expect("Unable to get pipeline bus");
        let s = sender.clone();
        gtk::timeout_add(250, move || {
            while bus.have_pending() {
                bus.pop().map(|message| {
                    //debug!("new message {:?}", message);
                    Self::parse_bus_message(&message, s.clone(), ct.clone());
                });
            }
            Continue(true)
        });

        let recorderbin = Arc::new(Mutex::new(None));

        let pipeline = Self {
            pipeline,
            uridecodebin,
            audioconvert,
            tee,
            audio_queue,
            volume,
            autoaudiosink,
            file_queue,
            recorderbin,
            file_srcpad,
            file_blockprobe_id: None,
            current_title,
            sender,
        };

        pipeline
    }

    pub fn set_state(&mut self, state: gstreamer::State) {
        if state == gstreamer::State::Null {
            self.sender.send(GstreamerMessage::PlaybackStateChanged(PlaybackState::Stopped)).unwrap();
        }

        let _ = self.pipeline.set_state(state);
    }

    pub fn new_source_uri(&mut self, source: &str) {
        debug!("Stop pipeline...");
        let _ = self.pipeline.set_state(State::Null);

        debug!("Set new source uri...");
        self.uridecodebin.set_property("uri", &source).unwrap();

        debug!("Start pipeline...");
        let _ = self.pipeline.set_state(State::Playing);
    }

    pub fn start_recording(&mut self, path: PathBuf) {
        debug!("Start recording to \"{:?}\"...", path);

        // We need to set an offset, otherwise the length of the recorded song would be wrong.
        // Get current clock time and calculate offset
        let clock = self.pipeline.get_clock().expect("Could not get gstreamer pipeline clock");
        debug!("Clock time: {}", clock.get_time());
        let offset = -(clock.get_time().nseconds().unwrap() as i64);
        self.file_srcpad.set_offset(offset);

        debug!("Destroy old recorderbin...");
        if self.recorderbin.lock().unwrap().is_some() {
            self.recorderbin.lock().unwrap().take().unwrap().destroy();
        } else {
            debug!("No recorderbin available - nothing to destroy");
        }

        debug!("Create new recorderbin");
        let recorderbin = RecorderBin::new(self.get_current_song_title(), path, self.pipeline.clone(), &self.file_srcpad);
        *self.recorderbin.lock().unwrap() = Some(recorderbin);

        // Remove block probe id, if available
        debug!("Remove block probe...");
        match self.file_blockprobe_id.take() {
            Some(id) => self.file_srcpad.remove_probe(id),
            None => (),
        }

        debug!("Everything ok.");
    }

    pub fn stop_recording(&mut self, save_song: bool) -> Option<Song> {
        debug!("Stop recording... (save song: {})", save_song);

        if self.recorderbin.lock().unwrap().is_some() {
            let rbin = self.recorderbin.clone();
            if save_song {
                let file_id = self
                    .file_srcpad
                    .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, move |_, _| {
                        // Dataflow is blocked
                        debug!("Push EOS into recorderbin sinkpad...");
                        let sinkpad = rbin.lock().unwrap().clone().unwrap().gstbin.get_static_pad("sink").unwrap();
                        sinkpad.send_event(gstreamer::Event::new_eos().build());

                        gstreamer::PadProbeReturn::Ok
                    })
                    .unwrap();

                // We need the padprobe id later to remove the block probe
                self.file_blockprobe_id = Some(file_id);

                // Create song and return it
                let song = self.recorderbin.lock().unwrap().clone().unwrap().stop();
                return Some(song);
            } else {
                debug!("Discard recorded data");
                let recorderbin = self.recorderbin.lock().unwrap().take().unwrap();
                fs::remove_file(&recorderbin.song_path).expect("Could not delete recorded data");
                recorderbin.destroy();
                return None;
            }
        } else {
            debug!("No recorderbin available - nothing to stop");
            return None;
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recorderbin.lock().unwrap().is_some()
    }

    pub fn get_current_song_title(&self) -> String {
        self.current_title.lock().unwrap().clone()
    }

    fn parse_bus_message(message: &gstreamer::Message, sender: Sender<GstreamerMessage>, current_title: Arc<Mutex<String>>) {
        match message.view() {
            gstreamer::MessageView::Tag(tag) => {
                tag.get_tags().get::<gstreamer::tags::Title>().map(|t| {
                    let new_title = t.get().unwrap().to_string();

                    // only send message if song title really have changed.
                    if *current_title.lock().unwrap() != new_title {
                        *current_title.lock().unwrap() = new_title.clone();
                        sender.send(GstreamerMessage::SongTitleChanged(new_title)).unwrap();
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

                sender.send(GstreamerMessage::PlaybackStateChanged(playback_state)).unwrap();
            }
            gstreamer::MessageView::Element(element) => {
                let structure = element.get_structure().unwrap();
                if structure.get_name() == "GstBinForwarded" {
                    let message: gstreamer::message::Message = structure.get("message").unwrap();
                    if let gstreamer::MessageView::Eos(_) = &message.view() {
                        // recorderbin got EOS which means the current song got successfully saved.
                        debug!("Recorderbin received EOS...");
                        sender.send(GstreamerMessage::RecordingStopped).unwrap();
                    }
                }
            }
            gstreamer::MessageView::Error(err) => {
                let msg = err.get_error().to_string();
                warn!("Gstreamer Error: {:?}", msg);
                sender.send(GstreamerMessage::PlaybackStateChanged(PlaybackState::Failure(msg))).unwrap();
            }
            _ => (),
        };
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                      //
//  # RecorderBin                                                                                       //
//                                                                                                      //
//    --------------------------------------------------------------                                    //
//   |                  -----------       --------      ----------  |                                   //
//   | ( ghostpad ) -> | vorbisenc | ->  | oggmux | -> | filesink | |                                   //
//   |                  -----------       --------      ----------  |                                   //
//    --------------------------------------------------------------                                    //
//                                                                                                      //
/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Clone)]
struct RecorderBin {
    pub gstbin: Bin,
    pipeline: Pipeline,

    ghostpad: GhostPad,
    vorbisenc: Element,
    oggmux: Element,
    filesink: Element,

    song_title: String,
    pub song_path: PathBuf,
    song_timestamp: SystemTime,
}

impl RecorderBin {
    pub fn new(song_title: String, song_path: PathBuf, pipeline: Pipeline, srcpad: &Pad) -> Self {
        // Create elements
        let vorbisenc = ElementFactory::make("vorbisenc", "vorbisenc").unwrap();
        let oggmux = ElementFactory::make("oggmux", "oggmux").unwrap();
        let filesink = ElementFactory::make("filesink", "filesink").unwrap();
        filesink.set_property("location", &song_path.to_str().unwrap()).unwrap();

        // Create bin itself
        let bin = Bin::new("bin");
        bin.set_property("message-forward", &true).unwrap();

        // Add elements to bin and link them
        bin.add(&vorbisenc).unwrap();
        bin.add(&oggmux).unwrap();
        bin.add(&filesink).unwrap();
        Element::link_many(&[&vorbisenc, &oggmux, &filesink]).unwrap();

        // Add bin to pipeline
        pipeline.add(&bin).expect("Could not add recorderbin to pipeline");

        // Link file_srcpad with vorbisenc sinkpad using a ghostpad
        let vorbisenc_sinkpad = vorbisenc.get_static_pad("sink").unwrap();
        let ghostpad = gstreamer::GhostPad::new("sink", &vorbisenc_sinkpad).unwrap();
        bin.add_pad(&ghostpad).unwrap();
        bin.sync_state_with_parent().unwrap();
        srcpad.link(&ghostpad).expect("Queue src pad cannot linked to vorbisenc sinkpad");

        // Set song timestamp so we can check the duration later
        let song_timestamp = SystemTime::now();

        Self {
            gstbin: bin,
            pipeline,
            ghostpad,
            vorbisenc,
            oggmux,
            filesink,
            song_title,
            song_path,
            song_timestamp,
        }
    }

    pub fn stop(&self) -> Song {
        let now = SystemTime::now();
        let duration = now.duration_since(self.song_timestamp).unwrap();

        Song::new(&self.song_title, self.song_path.clone(), duration)
    }

    pub fn destroy(&self) {
        self.pipeline.remove(&self.gstbin).unwrap();
        self.gstbin.set_state(State::Null).unwrap();
    }
}
