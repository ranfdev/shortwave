use gstreamer::prelude::*;
use gstreamer::{Bin, Bus, Element, ElementFactory, Pad, PadProbeId, Pipeline, State};

//////////////////////////////////////////////////////////////////////////////////////////////////////////
//                                                                                                      //
//  # Gstreamer Pipeline                                                                                //
//                                            -----      --------       ------------                    //
//                                           |     | -> | queue [1] -> | muxsinkbin |                   //
//    --------------      --------------     |     |     --------       ------------                    //
//   | uridecodebin | -> | audioconvert | -> | tee |                                                    //
//    --------------      --------------     |     |     --------       --------      ---------------   //
//                                           |     | -> | queue [2] -> | volume | -> | autoaudiosink |  //
//                                            -----      --------       --------      ---------------   //
//                                                                                                      //
//                                                                                                      //
//                                                                                                      //
//  We use the the file_srcpad[1] to block the dataflow, so we can change the muxsinkbin.               //
//  The dataflow gets blocked when the song changes.                                                    //
//                                                                                                      //
//                                                                                                      //
//  # muxsinkbin:  (gstreamer Bin)                                                                      //
//    --------------------------------------------------------------                                    //
//   |                  -----------       --------      ----------  |                                   //
//   | ( ghostpad ) -> | vorbisenc | ->  | oggmux | -> | filesink | |                                   //
//   |                  -----------       --------      ----------  |                                   //
//    --------------------------------------------------------------                                    //
//                                                                                                      //
//////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub struct PlayerBackend {
    pipeline: Pipeline,

    uridecodebin: Element,
    audioconvert: Element,
    tee: Element,

    audio_queue: Element,
    volume: Element,
    autoaudiosink: Element,

    file_queue: Element,
    muxsinkbin: Option<Bin>,
    file_srcpad: Pad,
    file_blockprobe_id: Option<PadProbeId>,
}

impl PlayerBackend {
    pub fn new() -> Self {
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

        let mut pipeline = Self {
            pipeline,
            uridecodebin,
            audioconvert,
            tee,
            audio_queue,
            volume,
            autoaudiosink,
            file_queue,
            muxsinkbin: None,
            file_srcpad,
            file_blockprobe_id: None,
        };

        pipeline.create_muxsinkbin("/dev/null");
        pipeline
    }

    pub fn set_state(&self, state: gstreamer::State) {
        let _ = self.pipeline.set_state(state);
    }

    pub fn set_volume(&self, v: f64) {
        debug!("Set volume: {}", v);
        self.volume.set_property("volume", &v).unwrap();
    }

    pub fn get_pipeline_bus(&self) -> Bus {
        self.pipeline.get_bus().expect("Unable to get pipeline bus")
    }

    pub fn block_dataflow(&mut self) {
        // File branch
        let muxsinkbin = self.muxsinkbin.clone();
        let file_id = self
            .file_srcpad
            .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, move |_, _| {
                // Dataflow is blocked
                debug!("Pad is blocked now.");

                debug!("Push EOS into muxsinkbin sinkpad...");
                let sinkpad = muxsinkbin.clone().unwrap().get_static_pad("sink").unwrap();
                sinkpad.send_event(gstreamer::Event::new_eos().build());

                gstreamer::PadProbeReturn::Ok
            })
            .unwrap();

        // We need the padprobe id later to remove the block probe
        self.file_blockprobe_id = Some(file_id);
    }

    pub fn new_source_uri(&mut self, source: &str) {
        debug!("Stop pipeline...");
        let _ = self.pipeline.set_state(State::Null);

        debug!("Set new source uri...");
        self.uridecodebin.set_property("uri", &source).unwrap();

        debug!("Start pipeline...");
        let _ = self.pipeline.set_state(State::Playing);
    }

    pub fn new_filesink_location(&mut self, location: &str) {
        debug!("Update filesink location to \"{}\"...", location);

        debug!("Destroy old muxsinkbin");
        let muxsinkbin = self.muxsinkbin.take().unwrap();
        let _ = muxsinkbin.set_state(State::Null);
        self.pipeline.remove(&muxsinkbin).unwrap();

        debug!("Create new muxsinkbin");
        self.create_muxsinkbin(location);

        debug!("Remove block probe..."); //TODO: Fix crash here... (called `Option::unwrap()` on a `None) 169
        self.file_srcpad.remove_probe(self.file_blockprobe_id.take().unwrap());

        debug!("Everything ok.");
    }

    fn create_muxsinkbin(&mut self, location: &str) {
        // Create vorbisenc
        let vorbisenc = ElementFactory::make("vorbisenc", "vorbisenc").unwrap();

        // Create oggmux
        let oggmux = ElementFactory::make("oggmux", "oggmux").unwrap();

        // Create filesink
        let filesink = ElementFactory::make("filesink", "filesink").unwrap();
        filesink.set_property("location", &location).unwrap();

        // Create bin
        let bin = Bin::new("bin");
        bin.set_property("message-forward", &true).unwrap();

        // Add elements to bin and link them
        bin.add(&vorbisenc).unwrap();
        bin.add(&oggmux).unwrap();
        bin.add(&filesink).unwrap();
        Element::link_many(&[&vorbisenc, &oggmux, &filesink]).unwrap();

        // Add bin to pipeline
        self.pipeline.add(&bin).unwrap();

        // Link queue src pad with vorbisenc sinkpad using a ghostpad
        let vorbisenc_sinkpad = vorbisenc.get_static_pad("sink").unwrap();

        let ghostpad = gstreamer::GhostPad::new("sink", &vorbisenc_sinkpad).unwrap();
        bin.add_pad(&ghostpad).unwrap();
        bin.sync_state_with_parent().unwrap();

        if self.file_srcpad.link(&ghostpad) != gstreamer::PadLinkReturn::Ok {
            warn!("Queue src pad cannot linked to vorbisenc sinkpad");
        }

        self.muxsinkbin = Some(bin);
    }
}

/////////////////////////////////////////////////////////////////////////////////////
//                                                                                 //
//  # Gstreamer Pipeline                                                           //
//                                                                                 //
//    ---------      -----------      -----------      --------      ----------    //
//   | filesrc | -> | decodebin | -> | vorbisenc | -> | oggmux | -> | filesink |   //
//    ---------      -----------      -----------      --------      ----------    //
//                                                                                 //
//  We need a separate pipeline for exporting a song, otherwise the duration would //
//  be wrong. For reference:                                                       //
//                                                                                 //
//  http://gstreamer-devel.966125.n4.nabble.com/Dynamically-change-filesink-File-duration-problem-td4689353.html
//                                                                                 //
/////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ExportBackend {
    pipeline: Pipeline,
    path: String,
    export_path: String,
}

impl ExportBackend {
    pub fn new(path: &str, export_path: &str) -> Self {
        let pipeline = Pipeline::new("export_pipeline");

        let filesrc = ElementFactory::make("filesrc", "filesrc").unwrap();
        let decodebin = ElementFactory::make("decodebin", "decodebin").unwrap();
        let vorbisenc = ElementFactory::make("vorbisenc", "vorbisenc").unwrap();
        let oggmux = ElementFactory::make("oggmux", "oggmux").unwrap();
        let filesink = ElementFactory::make("filesink", "filesink").unwrap();

        pipeline.add_many(&[&filesrc, &decodebin, &vorbisenc, &oggmux, &filesink]).unwrap();
        Element::link_many(&[&vorbisenc, &oggmux, &filesink]).unwrap();
        Element::link_many(&[&filesrc, &decodebin]).unwrap();

        let vorbis = vorbisenc.clone();
        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = vorbis.get_static_pad("sink").expect("Failed to get static sink pad from convert");
            let _ = src_pad.link(&sink_pad);
        });

        filesrc.set_property("location", &path).unwrap();
        filesink.set_property("location", &export_path).unwrap();

        Self {
            pipeline,
            path: path.to_string(),
            export_path: export_path.to_string(),
        }
    }

    pub fn start(&self) {
        debug!("* Export song **");
        debug!("Cached song path: {}", self.path);
        debug!("Export song path: {}", self.export_path);
        let _ = self.pipeline.set_state(State::Playing);
    }
}
