use stopwatch::Stopwatch;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct Song {
    pub title: String,
    pub title_hash: String,
    pub path: String,
    pub duration: Stopwatch, 
}

impl Song {
    pub fn new(title: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        title.hash(&mut hasher);
        let title_hash = hasher.finish().to_string();
        let path = format!("{}/{}.ogg", glib::get_user_cache_dir().unwrap().to_str().unwrap(), title_hash);
        let duration = Stopwatch::start_new();

        Self {
            title: title.to_string(),
            title_hash,
            path,
            duration,
        }
    }

    pub fn finish(&mut self){
        self.duration.stop();
    }

    pub fn delete(&mut self){
        self.finish();
        // TODO: implement
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Song) -> bool {
        self.title == other.title
    }
}
