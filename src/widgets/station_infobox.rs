use gtk::prelude::*;
use rustio::Station;

use crate::song::Song;
use crate::widgets::song_row::SongRow;

pub struct StationInfobox {
    pub widget: gtk::Box,
    builder: gtk::Builder,
}

impl StationInfobox {
    pub fn new() -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_infobox.ui");
        let widget: gtk::Box = builder.get_object("station_infobox").unwrap();

        Self {
            widget,
            builder,
        }
    }

    pub fn set_station(&self, station: &Station){
        let codec_label: gtk::Label = self.builder.get_object("codec_label").unwrap();
        let homepage_label: gtk::Label = self.builder.get_object("homepage_label").unwrap();
        let tags_label: gtk::Label = self.builder.get_object("tags_label").unwrap();
        let language_label: gtk::Label = self.builder.get_object("language_label").unwrap();

        codec_label.set_text(&station.codec);
        homepage_label.set_markup(&format!("<a href=\"{}\">{}</a>", station.homepage, station.homepage));
        tags_label.set_text(&station.tags);
        language_label.set_text(&station.language);
    }
}
