use chrono::NaiveTime;
use glib::Sender;
use gtk::prelude::*;
use libhandy::{ActionRow, ActionRowExt};
use open;

use std::path::PathBuf;

use crate::app::Action;
use crate::song::Song;

pub struct SongRow {
    pub widget: ActionRow,
    song: Song,
    button_stack: gtk::Stack,
    save_button: gtk::Button,
    open_button: gtk::Button,
}

impl SongRow {
    pub fn new(_sender: Sender<Action>, song: Song) -> Self {
        let widget = ActionRow::new();
        widget.set_title(&song.title);
        widget.set_subtitle(&Self::format_duration(song.duration.as_secs()));
        widget.set_icon_name("");

        let button_stack = gtk::Stack::new();
        widget.add_action(&button_stack);

        let save_button = gtk::Button::new();
        save_button.set_relief(gtk::ReliefStyle::None);
        save_button.set_valign(gtk::Align::Center);
        let save_image = gtk::Image::new_from_icon_name("document-save-symbolic", gtk::IconSize::__Unknown(4));
        save_button.add(&save_image);
        button_stack.add_named(&save_button, "save");

        let open_button = gtk::Button::new();
        open_button.set_relief(gtk::ReliefStyle::None);
        open_button.set_valign(gtk::Align::Center);
        let open_image = gtk::Image::new_from_icon_name("media-playback-start-symbolic", gtk::IconSize::__Unknown(4));
        open_button.add(&open_image);
        button_stack.add_named(&open_button, "open");

        widget.show_all();

        let row = Self {
            widget,
            song,
            button_stack,
            save_button,
            open_button,
        };

        row.setup_signals();
        row
    }

    fn setup_signals(&self) {
        let song = self.song.clone();
        let widget = self.widget.clone();
        let button_stack = self.button_stack.clone();
        self.save_button.connect_clicked(move |_| {
            let mut path = PathBuf::from(glib::get_user_special_dir(glib::UserDirectory::Music).unwrap());
            path.push(&Song::simplify_title(song.title.clone()));
            match song.save_as(path) {
                Ok(()) => {
                    widget.set_subtitle("Saved");
                    button_stack.set_visible_child_name("open");
                }
                Err(err) => widget.set_subtitle(&err.to_string()),
            };
        });

        let song = self.song.clone();
        self.open_button.connect_clicked(move |_| {
            open::that(song.path.clone()).expect("Could not play song");
        });
    }

    // stolen from gnome-podcasts
    // https://gitlab.gnome.org/haecker-felix/podcasts/blob/2f8a6a91f87d7fa335a954bbaf2f70694f32f6dd/podcasts-gtk/src/widgets/player.rs#L168
    fn format_duration(seconds: u64) -> String {
        let time = NaiveTime::from_num_seconds_from_midnight(seconds as u32, 0);

        if seconds >= 3600 {
            time.format("%T").to_string()
        } else {
            time.format("%M∶%S").to_string()
        }
    }
}
