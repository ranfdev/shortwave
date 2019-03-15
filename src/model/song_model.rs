use gio::prelude::ListStoreExtManual;
use gio::prelude::*;
use glib::prelude::*;

use crate::song::Song;
use crate::song_object::SongObject;

#[derive(Clone, Debug)]
pub struct SongModel {
    pub model: gio::ListStore,
}

impl SongModel {
    pub fn new() -> Self {
        let model = gio::ListStore::new(SongObject::static_type());

        Self { model, sorting, order }
    }

    pub fn add_song(&mut self, song: Song) {
        if !self.index(&song).is_some() {
            let object = SongObject::new(song.clone());
            let sorting = self.sorting.clone();
            let order = self.order.clone();
            self.model.insert(&object);
        }
    }

    pub fn remove_song(&mut self, song: &Song) {
        self.index(song).map(|index| self.model.remove(index));
    }

    fn index(&self, song: &Song) -> Option<u32> {
        for i in 0..self.model.get_n_items() {
            let gobject = self.model.get_object(i).unwrap();
            let song_object = gobject.downcast_ref::<SongObject>().expect("SongObject is of wrong type");
            let s = song_object.to_song();

            if &s == song {
                return Some(i);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.model.remove_all();
    }
}
