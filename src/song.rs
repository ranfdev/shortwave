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
        debug!("New song: \"{}\", {:?}", title, path);

        Self {
            title: title.to_string(),
            path,
            duration,
        }
    }

    pub fn save_as(&self, path: PathBuf) -> std::io::Result<()> {
        debug!("Save song \"{}\" as {:?}", self.title, path);

        fs::copy(&self.path, path)?;
        Ok(())
    }

    pub fn simplify_title(t: String) -> String {
        // remove special chars from title
        // if anybody knows a better way to do this, feel free to open a MR on GitLab :)
        let mut title = t.replace("/", "");
        title = title.replace("\\", "");
        title = title.replace(":", "");
        title = title.replace("<", "");
        title = title.replace(">", "");
        title = title.replace("\"", "");
        title = title.replace("|", "");
        title = title.replace("?", "");
        title = title.replace("*", "");
        title = title.replace(".", "");
        title
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Song) -> bool {
        self.title == other.title
    }
}
