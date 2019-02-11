use stopwatch::Stopwatch;
use uuid::Uuid;

use std::fs;

use crate::player::gstreamer_backend::ExportBackend;

#[derive(Clone)]
pub struct Song {
    pub title: String,
    pub uuid: Uuid,
    pub path: String,
    pub duration: Stopwatch,
    pipeline: ExportBackend,
}

impl Song {
    pub fn new(title: &str) -> Self {
        // generate uuid for song
        let uuid = Uuid::new_v4();
        let path = format!("{}/{}.ogg", glib::get_user_cache_dir().unwrap().to_str().unwrap(), uuid);
        let duration = Stopwatch::start_new();

        // remove special chars from title
        // if anybody knows a better way to do this, feel free to open a MR on GitLab :)
        let mut export_title = title.clone().to_string();
        export_title = export_title.replace("/", "");
        export_title = export_title.replace("\\", "");
        export_title = export_title.replace(":", "");
        export_title = export_title.replace("<", "");
        export_title = export_title.replace(">", "");
        export_title = export_title.replace("\"", "");
        export_title = export_title.replace("|", "");
        export_title = export_title.replace("?", "");
        export_title = export_title.replace("*", "");
        let export_path = format!("{}/{}.ogg", glib::get_user_special_dir(glib::UserDirectory::Music).unwrap().to_str().unwrap(), export_title);
        let pipeline = ExportBackend::new(&path, &export_path);

        debug!("Cache song \"{}\" -> {}", title, path);

        Self {
            title: title.to_string(),
            uuid,
            path,
            duration,
            pipeline,
        }
    }

    pub fn finish(&mut self) {
        self.duration.stop();
    }

    pub fn delete(&mut self) {
        self.finish();
        fs::remove_file(&self.path).expect("Could not delete song.");
    }

    pub fn export(&self) {
        self.pipeline.start();
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Song) -> bool {
        self.title == other.title
    }
}
