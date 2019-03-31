use glib::Sender;
use gtk::prelude::*;
use libhandy::Dialog;
use rustio::Station;

use crate::app::Action;
use crate::library::Library;

pub struct StationDialog {
    pub widget: Dialog,
    station: Station,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    codec_label: gtk::Label,
    homepage_label: gtk::Label,
    tags_label: gtk::Label,
    language_label: gtk::Label,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl StationDialog {
    pub fn new(sender: Sender<Action>, station: Station, window: &gtk::Window) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_dialog.ui");
        let widget: Dialog = builder.get_object("station_dialog").unwrap();

        let title_label: gtk::Label = builder.get_object("title_label").unwrap();
        let subtitle_label: gtk::Label = builder.get_object("subtitle_label").unwrap();
        let codec_label: gtk::Label = builder.get_object("codec_label").unwrap();
        let homepage_label: gtk::Label = builder.get_object("homepage_label").unwrap();
        let tags_label: gtk::Label = builder.get_object("tags_label").unwrap();
        let language_label: gtk::Label = builder.get_object("language_label").unwrap();

        // Show correct library action
        let library_action_stack: gtk::Stack = builder.get_object("library_action_stack").unwrap();
        if Library::contains_station(&station) {
            library_action_stack.set_visible_child_name("library-remove");
        } else {
            library_action_stack.set_visible_child_name("library-add");
        }

        widget.set_transient_for(window);

        let dialog = Self {
            widget,
            station,
            title_label,
            subtitle_label,
            codec_label,
            homepage_label,
            tags_label,
            language_label,
            builder,
            sender,
        };

        dialog.setup();
        dialog.setup_signals();
        dialog
    }

    fn setup(&self) {
        self.title_label.set_text(&self.station.name);
        let subtitle_text = &format!("{} {} · {} Votes", self.station.country, self.station.state, self.station.votes);
        self.subtitle_label.set_text(subtitle_text);

        if self.station.codec != "" {
            self.codec_label.set_text(&self.station.codec);
        }
        if self.station.homepage != "" {
            self.homepage_label.set_markup(&format!("<a href=\"{}\">{}</a>", self.station.homepage, self.station.homepage));
        }
        if self.station.tags != "" {
            self.tags_label.set_text(&self.station.tags);
        }
        if self.station.language != "" {
            self.language_label.set_text(&self.station.language);
        }
    }

    pub fn show(&self) {
        self.widget.set_visible(true);
    }

    fn setup_signals(&self) {
        // remove_button
        let library_action_stack: gtk::Stack = self.builder.get_object("library_action_stack").unwrap();
        let remove_button: gtk::Button = self.builder.get_object("remove_button").unwrap();
        let sender = self.sender.clone();
        let station = self.station.clone();
        remove_button.connect_clicked(move |_| {
            sender.send(Action::LibraryRemoveStations(vec![station.clone()])).unwrap();
            library_action_stack.set_visible_child_name("library-add");
        });

        // add_button
        let library_action_stack: gtk::Stack = self.builder.get_object("library_action_stack").unwrap();
        let add_button: gtk::Button = self.builder.get_object("add_button").unwrap();
        let sender = self.sender.clone();
        let station = self.station.clone();
        add_button.connect_clicked(move |_| {
            sender.send(Action::LibraryAddStations(vec![station.clone()])).unwrap();
            library_action_stack.set_visible_child_name("library-remove");
        });
    }
}
