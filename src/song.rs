use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
    pub duration: Duration,
}

impl Song {
    pub fn new(title: &str, path: PathBuf, duration: Duration) -> Self {
        Self {
            title: title.to_string(),
            path,
            duration,
        }
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
