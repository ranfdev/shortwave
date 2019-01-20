use stopwatch::Stopwatch;
use gstreamer::prelude::*;
use gstreamer::{Element, Pipeline, State, ElementFactory, ClockTime};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time;

use crate::gstreamer_backend::ExportBackend;

#[derive(Clone)]
pub struct Song {
    pub title: String,
    pub title_hash: String,
    pub path: String,
    pub duration: Stopwatch,
    pipeline: ExportBackend,
}

impl Song {
    pub fn new(title: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        title.hash(&mut hasher);
        let title_hash = hasher.finish().to_string();
        let path = format!("{}/{}.ogg", glib::get_user_cache_dir().unwrap().to_str().unwrap(), title_hash);
        let duration = Stopwatch::start_new();

        let export_path = format!("{}/{}.ogg", glib::get_user_special_dir(glib::UserDirectory::Music).unwrap().to_str().unwrap(), title);
        let pipeline = ExportBackend::new(&path, &export_path);

        Self {
            title: title.to_string(),
            title_hash,
            path,
            duration,
            pipeline,
        }
    }

    pub fn finish(&mut self){
        self.duration.stop();
    }

    pub fn delete(&mut self){
        self.finish();
        // TODO: implement
    }

    pub fn export(&self){
        self.pipeline.start();
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Song) -> bool {
        self.title == other.title
    }
}
