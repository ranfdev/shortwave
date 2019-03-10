use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use rusqlite::Connection;
use rustio::{Client, Station};

use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::result::Result;
use std::thread;

use crate::app::Action;
use crate::config;
use crate::station_model::StationModel;
use crate::station_model::{Order, Sorting};
use crate::station_object::StationObject;
use crate::widgets::station_listbox::StationListBox;

lazy_static! {
    static ref LIBRARY_PATH: PathBuf = {
        let mut path = glib::get_user_data_dir().unwrap();
        path.push(config::NAME);
        path.push("library.json");
        path
    };
}

pub struct Library {
    pub widget: gtk::Box,
    library_model: RefCell<StationModel>,

    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl Library {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/library.ui");
        let widget: gtk::Box = builder.get_object("library").unwrap();
        let content_box: gtk::Box = builder.get_object("content_box").unwrap();

        let library_model = RefCell::new(StationModel::new());
        let station_listbox = StationListBox::new(sender.clone());
        station_listbox.bind_model(&library_model.borrow());
        content_box.add(&station_listbox.widget);

        let logo_image: gtk::Image = builder.get_object("logo_image").unwrap();
        logo_image.set_from_icon_name(Some(format!("{}-symbolic", config::APP_ID).as_str()), gtk::IconSize::__Unknown(128));
        let welcome_text: gtk::Label = builder.get_object("welcome_text").unwrap();
        welcome_text.set_text(format!("Welcome to {}", config::NAME).as_str());

        let library = Self {
            widget,
            library_model,
            builder,
            sender,
        };

        library.setup_signals();

        // Read stations
        match Self::read(LIBRARY_PATH.to_path_buf()) {
            Ok(stations) => library.add_stations(stations),
            Err(error) => {
                let message = format!("Could not read library data: {}", error.to_string());
                library.sender.send(Action::ViewShowNotification(message)).unwrap();
            }
        };

        library
    }

    pub fn add_stations(&self, stations: Vec<Station>) {
        debug!("Add {} station(s)", stations.len());
        for station in stations {
            self.library_model.borrow_mut().add_station(station.clone());
        }
    }

    pub fn remove_stations(&self, stations: Vec<Station>) {
        debug!("Remove {} station(s)", stations.len());
        for station in stations {
            self.library_model.borrow_mut().remove_station(&station);
        }
    }

    pub fn set_sorting(&self, sorting: Sorting, order: Order) {
        self.library_model.borrow_mut().set_sorting(sorting, order);
    }

    pub fn to_vec(&self) -> Vec<Station> {
        Self::model_to_vec(&self.library_model.borrow().model)
    }

    fn model_to_vec(model: &gio::ListStore) -> Vec<Station> {
        let mut stations = Vec::new();
        for i in 0..model.get_n_items() {
            let gobject = model.get_object(i).unwrap();
            let station_object = gobject.downcast_ref::<StationObject>().expect("StationObject is of wrong type");
            stations.insert(0, station_object.to_station());
        }
        stations
    }

    fn setup_signals(&self) {
        let sender = self.sender.clone();
        self.library_model.borrow().model.connect_items_changed(move |model, pos, removed, added| {
            // Check if data got changed
            if removed == 1 || added == 1 {
                // Convert gio::ListStore into Vec<Station>
                let stations = Self::model_to_vec(model);

                // Write new data to disk
                match Self::write(stations, LIBRARY_PATH.to_path_buf()) {
                    Ok(()) => (),
                    Err(error) => {
                        let message = format!("Could not write library data: {}", error.to_string());
                        sender.send(Action::ViewShowNotification(message)).unwrap();
                    }
                };
            }
        });
    }

    pub fn write(stations: Vec<Station>, path: PathBuf) -> Result<(), LibraryError> {
        debug!("Write library data to: {:?}", path);

        // Convert Vec<Station> into text
        let data = serde_json::to_string(&stations)?;

        // Create missing folders, if necessary
        let mut fpath = path.clone();
        fpath.pop();
        fs::create_dir_all(fpath)?;

        // Write the actual data
        fs::write(path, data)?;
        Ok(())
    }

    pub fn read(path: PathBuf) -> Result<Vec<Station>, LibraryError> {
        debug!("Read library data from: {:?}", path);

        if path.extension().is_some() && path.extension().unwrap().to_str() == Some("json") {
            // New Shortwave library format (.json)
            let data = fs::read_to_string(path)?;
            let stations: Vec<Station> = serde_json::from_str(&data)?;
            Ok(stations)
        } else {
            // Old Gradio library format (.db)
            let mut result = Vec::new();
            let mut client = Client::new("http://www.radio-browser.info");
            let connection = Connection::open(path.clone())?;
            let mut stmt = connection.prepare("SELECT station_id FROM library;")?;
            let mut rows = stmt.query(&[])?;

            while let Some(result_row) = rows.next() {
                let row = result_row.unwrap();
                let station_id: u32 = row.get(0);

                match client.get_station_by_id(station_id)? {
                    Some(station) => {
                        info!("Found Station: {}", station.name);
                        result.insert(0, station);
                    }
                    None => warn!("Could not fetch station with ID {}", station_id),
                }
            }
            Ok(result)
        }
    }

    pub fn contains_station(station: &Station) -> bool {
        match Self::read(LIBRARY_PATH.to_path_buf()) {
            Ok(stations) => {
                for s in stations {
                    if &s == station {
                        return true;
                    }
                }
                return false;
            }
            Err(_) => {
                return false;
            }
        };
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum LibraryError {
        Io(err: io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
        Sqlite(err: rusqlite::Error) {
            from()
            description("sqlite error")
            display("Gradio database error: {}", err)
            cause(err)
        }
        Restson(err: restson::Error) {
            from()
            description("restson error")
            display("Network error: {}", err)
            cause(err)
        }
        Serde(err: serde_json::error::Error) {
            from()
            description("serde error")
            display("Parser error: {}", err)
            cause(err)
        }
    }
}
