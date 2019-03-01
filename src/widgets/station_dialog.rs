use gtk::prelude::*;
use libhandy::Dialog;
use rustio::Station;

pub struct StationDialog {
    pub widget: Dialog,

    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    codec_label: gtk::Label,
    homepage_label: gtk::Label,
    tags_label: gtk::Label,
    language_label: gtk::Label,
}

impl StationDialog {
    pub fn new(station: &Station, window: &gtk::Window) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_dialog.ui");
        let widget: Dialog = builder.get_object("station_dialog").unwrap();

        let title_label: gtk::Label = builder.get_object("title_label").unwrap();
        let subtitle_label: gtk::Label = builder.get_object("subtitle_label").unwrap();
        let codec_label: gtk::Label = builder.get_object("codec_label").unwrap();
        let homepage_label: gtk::Label = builder.get_object("homepage_label").unwrap();
        let tags_label: gtk::Label = builder.get_object("tags_label").unwrap();
        let language_label: gtk::Label = builder.get_object("language_label").unwrap();

        widget.set_transient_for(window);

        let dialog = Self {
            widget,
            title_label,
            subtitle_label,
            codec_label,
            homepage_label,
            tags_label,
            language_label,
        };

        dialog.reset();
        dialog.set_station(&station);
        dialog
    }

    pub fn show(&self) {
        self.widget.set_visible(true);
    }

    fn set_station(&self, station: &Station) {
        self.reset();

        self.title_label.set_text(&station.name);
        self.subtitle_label.set_text(&format!("{} {} · {} Votes", station.country, station.state, station.votes));

        if station.codec != "" {
            self.codec_label.set_text(&station.codec);
        }
        if station.homepage != "" {
            self.homepage_label.set_markup(&format!("<a href=\"{}\">{}</a>", station.homepage, station.homepage));
        }
        if station.tags != "" {
            self.tags_label.set_text(&station.tags);
        }
        if station.language != "" {
            self.language_label.set_text(&station.language);
        }
    }

    fn reset(&self) {
        self.codec_label.set_text("—");
        self.homepage_label.set_text("—");
        self.tags_label.set_text("—");
        self.language_label.set_text("—");
    }
}
