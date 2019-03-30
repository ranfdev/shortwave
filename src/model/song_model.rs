use gio::prelude::*;
use glib::prelude::*;

use std::fs;

use crate::model::ObjectWrapper;
use crate::song::Song;

#[derive(Clone)]
pub struct SongModel {
    pub model: gio::ListStore,
    max_len: u32,

    current_song: Option<Song>,
    current_song_timestamp: Option<u32>,
}

impl SongModel {
    pub fn new(max_len: u32) -> Self {
        let model = gio::ListStore::new(ObjectWrapper::static_type());

        let current_song = None;
        let current_song_timestamp = None;

        Self {
            model,
            max_len,
            current_song,
            current_song_timestamp,
        }
    }

    pub fn add_song(&mut self, song: Song) {
        // Check if song does not exist yet
        if !self.index(&song).is_some() {
            // Ensure max length
            if self.model.get_n_items() >= self.max_len {
                let rm_song = self.get_song(self.model.get_n_items() - 1);
                self.remove_song(&rm_song).unwrap();
            }

            let object = ObjectWrapper::new(song.clone());
            self.model.insert(0, &object);
        }
    }

    pub fn remove_song(&mut self, song: &Song) -> std::io::Result<()> {
        fs::remove_file(&song.path)?;
        self.index(song).map(|index| self.model.remove(index));
        Ok(())
    }

    fn index(&self, song: &Song) -> Option<u32> {
        for i in 0..self.model.get_n_items() {
            let s = self.get_song(i);

            if &s == song {
                return Some(i);
            }
        }
        None
    }

    pub fn get_song(&self, index: u32) -> Song {
        let gobject = self.model.get_object(index).unwrap();
        let song_object = gobject.downcast_ref::<ObjectWrapper>().expect("ObjectWrapper is of wrong type");
        song_object.deserialize()
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        // Remove saved songs from disk
        for i in 0..self.model.get_n_items() {
            let s: Song = self.get_song(i);

            fs::remove_file(&s.path)?
        }

        self.model.remove_all();
        Ok(())
    }
}
