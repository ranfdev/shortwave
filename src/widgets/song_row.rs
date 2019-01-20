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
        save_button.set_relief(gtk::ReliefStyle::None);
        save_button.set_valign(gtk::Align::Center);
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
        let widget = self.widget.clone();
        let save_button = self.save_button.clone();
        self.save_button.connect_clicked(move |_| {
            song.export();
            widget.set_subtitle("Saved");
            save_button.set_visible(false);
        });
    }
}
