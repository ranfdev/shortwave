use gio::prelude::ListStoreExtManual;
use gio::prelude::*;
use glib::prelude::*;
use rustio::Station;

use crate::station_object::StationObject;

#[derive(Clone, Debug)]
pub enum Sorting {
    Name,
    Language,
    Country,
    State,
    Codec,
    Votes,
    Bitrate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
pub struct StationModel {
    pub model: gio::ListStore,
    sorting: Sorting,
    order: Order,

    iter_id: u32,
}

impl StationModel {
    pub fn new() -> Self {
        let model = gio::ListStore::new(StationObject::static_type());

        let sorting = Sorting::Name;
        let order = Order::Ascending;

        let iter_id = 0;

        Self { model, sorting, order, iter_id }
    }

    pub fn add_station(&mut self, station: Station) {
        let object = StationObject::new(station.clone());

        let sorting = self.sorting.clone();
        let order = self.order.clone();
        self.model.insert_sorted(&object, move |a, b| Self::station_cmp(a, b, sorting.clone(), order.clone()));
    }

    pub fn remove_station(&mut self, station: Station) {
        for i in 0..self.model.get_n_items() {
            let gobject = self.model.get_object(i).unwrap();
            let station_object = gobject.downcast_ref::<StationObject>().expect("StationObject is of wrong type");
            let s = station_object.to_station();

            if s == station {
                self.model.remove(i);
                break;
            }
        }
    }

    pub fn set_sorting(&mut self, sorting: Sorting, order: Order) {
        self.sorting = sorting.clone();
        self.order = order.clone();

        self.model.sort(move |a, b| Self::station_cmp(a, b, sorting.clone(), order.clone()));
    }

    pub fn clear(&mut self) {
        self.model.remove_all();
    }

    pub fn len(&self) -> u32 {
        self.model.get_n_items()
    }

    fn station_cmp(a: &gio::Object, b: &gio::Object, sorting: Sorting, order: Order) -> std::cmp::Ordering {
        let mut station_a: Station = a.downcast_ref::<StationObject>().unwrap().to_station();
        let mut station_b: Station = b.downcast_ref::<StationObject>().unwrap().to_station();

        if order == Order::Descending {
            let tmp = station_a;
            station_a = station_b;
            station_b = tmp;
        }

        match sorting {
            Sorting::Name => station_a.name.cmp(&station_b.name),
            Sorting::Language => station_a.language.cmp(&station_b.language),
            Sorting::Country => station_a.country.cmp(&station_b.country),
            Sorting::State => station_a.state.cmp(&station_b.state),
            Sorting::Codec => station_a.codec.cmp(&station_b.codec),
            Sorting::Votes => station_a.votes.parse::<i32>().unwrap().cmp(&station_b.votes.parse::<i32>().unwrap()),
            Sorting::Bitrate => station_a.bitrate.parse::<i32>().unwrap().cmp(&station_b.bitrate.parse::<i32>().unwrap()),
        }
    }
}

impl Iterator for StationModel {
    type Item = Station;

    fn next(&mut self) -> Option<Station> {
        let max = self.len();
        let mut result = None;

        if self.iter_id < max {
            let gobject = self.model.get_object(self.iter_id).unwrap();
            let station_object = gobject.downcast_ref::<StationObject>().expect("StationObject is of wrong type");
            result = Some(station_object.to_station());
            self.iter_id = self.iter_id + 1;
        }

        result
    }
}
