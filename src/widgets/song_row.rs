use gtk::prelude::*;
use libhandy::{ActionRow, ActionRowExt};

use crate::song::Song;

pub struct SongRow {
    pub widget: ActionRow,
    song: Song,
    save_button: gtk::Button,
}

impl SongRow {
    pub fn new(song: Song) -> Self {
        let widget = ActionRow::new();
        widget.set_title(&song.title);
        widget.set_subtitle(&song.duration.elapsed().as_secs().to_string()); //TODO: Display time correctly
        widget.set_icon_name("");

        let save_button = gtk::Button::new();
        let save_image = gtk::Image::new_from_icon_name("document-save-symbolic", 4);
        save_button.add(&save_image);
        widget.add_action(&save_button);
        widget.show_all();

        let row = Self {
            widget,
            song,
            save_button,
        };

        row.connect_signals();
        row
    }

    fn connect_signals(&self){
        let song = self.song.clone();
        self.save_button.connect_clicked(move |_|{
           let new_path = format!("{}/{}.ogg", glib::get_user_special_dir(glib::UserDirectory::Music).unwrap().to_str().unwrap(), song.title);
           std::fs::copy(song.clone().path, new_path).expect("Could not save song");
        });
    }
}
