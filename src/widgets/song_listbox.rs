use gtk::prelude::*;
use rustio::Station;

use crate::song::Song;
use crate::widgets::song_row::SongRow;

pub struct SongListBox {
    pub widget: gtk::Box,
    listbox: gtk::ListBox,

    pub current_station: Option<Station>,
    pub current_song: Option<Song>,

    max_length: usize,
    song_list: Vec<Song>,
    row_list: Vec<SongRow>,
}

impl SongListBox {
    pub fn new() -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/song_listbox.ui");
        let widget: gtk::Box = builder.get_object("song_listbox").unwrap();
        let listbox: gtk::ListBox = builder.get_object("listbox").unwrap();

        let current_station = None;
        let current_song = None;

        let max_length = 5;
        let song_list = Vec::new();
        let row_list = Vec::new();

        Self {
            widget,
            listbox,

            current_station,
            current_song,

            max_length,
            song_list,
            row_list,
        }
    }

    pub fn discard_current_song(&mut self) {
        self.current_song.take().map(|mut song| song.delete());
    }

    // returns 'true' if song have changed in comparsion to old song
    pub fn set_new_song(&mut self, song: Song) -> bool {
        // check if song have changed
        if self.current_song != Some(song.clone()) {
            self.current_song.take().map(|mut s| {
                // save current song
                s.finish();

                // create row and insert it into listbox
                let row = SongRow::new(s.clone());
                self.listbox.insert(&row.widget, 0);
                self.widget.set_visible(true);

                // insert widget (SongRow) and song into the lists
                self.row_list.insert(0, row);
                self.song_list.insert(0, s);
            });

            // set new current_song
            self.current_song = Some(song);

            // ensure max history length. Delete old song and remove it from listbox
            if self.song_list.len() > self.max_length {
                self.song_list.pop().map(|mut song| {
                    song.delete();
                    self.listbox.remove(&self.row_list.pop().unwrap().widget);
                });
            }
            return true;
        }
        false
    }

    pub fn delete_everything(&mut self) {
        self.discard_current_song();

        for song in &mut self.song_list {
            song.delete();
        }
        self.song_list.clear();
    }
}
