use glib::Sender;
use gtk::prelude::*;
use rustio::{Client, StationSearch};

use std::cell::RefCell;

use crate::app::Action;
use crate::model::StationModel;
use crate::widgets::station_listbox::StationListBox;

pub struct Search {
    pub widget: gtk::Box,
    result_model: RefCell<StationModel>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Search {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/search.ui");
        let widget: gtk::Box = builder.get_object("search").unwrap();

        let result_model = RefCell::new(StationModel::new());
        let results_box: gtk::Box = builder.get_object("results_box").unwrap();
        let station_listbox = StationListBox::new(sender.clone());
        station_listbox.bind_model(&result_model.borrow());
        results_box.add(&station_listbox.widget);

        let search = Self {
            widget,
            result_model,
            builder,
            sender,
        };

        search.setup_signals();
        search
    }

    pub fn search_for(&self, data: StationSearch) {
        debug!("search for: {:?}", data);

        let mut client = Client::new("http://www.radio-browser.info");
        let result = client.search(data).unwrap();

        self.result_model.borrow_mut().clear();
        for station in result {
            self.result_model.borrow_mut().add_station(station);
        }
    }

    fn setup_signals(&self) {
        let search_entry: gtk::SearchEntry = self.builder.get_object("search_entry").unwrap();
        let sender = self.sender.clone();
        search_entry.connect_search_changed(move |entry| {
            let data = StationSearch::search_for_name(entry.get_text().unwrap().to_string(), false, 100);
            sender.send(Action::SearchFor(data)).unwrap();
        });
    }
}
