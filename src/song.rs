use std::fs;
use std::io::Result;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
}

impl Song {
    pub fn new(title: &str, path: PathBuf) -> Self {
        Self { title: title.to_string(), path }
    }

    pub fn save_as(&self, path: PathBuf) -> std::io::Result<()> {
        fs::copy(&self.path, path)?;
        Ok(())
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Song) -> bool {
        self.title == other.title
    }
}
