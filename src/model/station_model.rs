use gio::prelude::ListStoreExtManual;
use gio::prelude::*;
use glib::prelude::*;
use rustio::Station;

use crate::model::ObjectWrapper;

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
}

impl StationModel {
    pub fn new() -> Self {
        let model = gio::ListStore::new(ObjectWrapper::static_type());

        let sorting = Sorting::Name;
        let order = Order::Ascending;

        Self { model, sorting, order }
    }

    pub fn add_station(&mut self, station: Station) {
        if !self.index(&station).is_some() {
            let object = ObjectWrapper::new(station.clone());
            let sorting = self.sorting.clone();
            let order = self.order.clone();
            self.model.insert_sorted(&object, move |a, b| Self::station_cmp(a, b, sorting.clone(), order.clone()));
        }
    }

    pub fn remove_station(&mut self, station: &Station) {
        self.index(station).map(|index| self.model.remove(index));
    }

    fn index(&self, station: &Station) -> Option<u32> {
        for i in 0..self.model.get_n_items() {
            let gobject = self.model.get_object(i).unwrap();
            let station_object = gobject.downcast_ref::<ObjectWrapper>().expect("ObjectWrapper is of wrong type");
            let s: Station = station_object.deserialize();

            if &s == station {
                return Some(i);
            }
        }
        None
    }

    pub fn set_sorting(&mut self, sorting: Sorting, order: Order) {
        self.sorting = sorting.clone();
        self.order = order.clone();

        self.model.sort(move |a, b| Self::station_cmp(a, b, sorting.clone(), order.clone()));
    }

    pub fn clear(&mut self) {
        self.model.remove_all();
    }

    fn station_cmp(a: &gio::Object, b: &gio::Object, sorting: Sorting, order: Order) -> std::cmp::Ordering {
        let mut station_a: Station = a.downcast_ref::<ObjectWrapper>().unwrap().deserialize();
        let mut station_b: Station = b.downcast_ref::<ObjectWrapper>().unwrap().deserialize();

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
