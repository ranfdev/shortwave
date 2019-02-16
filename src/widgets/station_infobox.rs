use gtk::prelude::*;
use rustio::Station;

pub struct StationInfobox {
    pub widget: gtk::Box,
    builder: gtk::Builder,

    codec_label: gtk::Label,
    homepage_label: gtk::Label,
    tags_label: gtk::Label,
    language_label: gtk::Label,
    votes_label: gtk::Label,
    location_label: gtk::Label,
}

impl StationInfobox {
    pub fn new() -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_infobox.ui");
        let widget: gtk::Box = builder.get_object("station_infobox").unwrap();

        let codec_label: gtk::Label = builder.get_object("codec_label").unwrap();
        let homepage_label: gtk::Label = builder.get_object("homepage_label").unwrap();
        let tags_label: gtk::Label = builder.get_object("tags_label").unwrap();
        let language_label: gtk::Label = builder.get_object("language_label").unwrap();
        let votes_label: gtk::Label = builder.get_object("votes_label").unwrap();
        let location_label: gtk::Label = builder.get_object("location_label").unwrap();

        Self {
            widget,
            builder,
            codec_label,
            homepage_label,
            tags_label,
            language_label,
            votes_label,
            location_label,
        }
    }

    pub fn set_station(&self, station: &Station) {
        self.reset();
        self.votes_label.set_text(&format!("{} Votes", station.votes));

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
        if !(station.country == "" && station.state == "") {
            self.location_label.set_text(&format!("{} {}", station.country, station.state));
        }
    }

    fn reset(&self) {
        self.codec_label.set_text("—");
        self.homepage_label.set_text("—");
        self.tags_label.set_text("—");
        self.language_label.set_text("—");
        self.location_label.set_text("—");
    }
}
