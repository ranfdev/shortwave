use gio::prelude::*;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use rustio::{Station, StationSearch};

use std::cell::RefCell;
use std::rc::Rc;

use crate::config;
use crate::library::Library;
use crate::player::{PlaybackState, Player};
use crate::search::Search;
use crate::station_model::{Order, Sorting};
use crate::window::{View, Window};

#[derive(Debug, Clone)]
pub enum Action {
    ViewShowSearch,
    ViewShowLibrary,
    ViewShowNotification(String),
    ViewRaise,
    ViewSetSorting(Sorting, Order),
    PlaybackSetStation(Station),
    PlaybackStart,
    PlaybackStop,
    LibraryImport,
    LibraryExport,
    LibraryAddStations(Vec<Station>),
    LibraryRemoveStations(Vec<Station>),
    SearchFor(StationSearch),
}

pub struct App {
    gtk_app: gtk::Application,

    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,

    window: Window,
    player: Player,
    library: Library,
    search: Search,
}

impl App {
    pub fn new() -> Rc<Self> {
        // Set custom style
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/de/haeckerfelix/Shortwave/gtk/style.css");
        gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &p, 500);

        let gtk_app = gtk::Application::new(config::APP_ID, gio::ApplicationFlags::FLAGS_NONE).unwrap();
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let window = Window::new(sender.clone());
        let player = Player::new(sender.clone());
        let library = Library::new(sender.clone());
        let search = Search::new(sender.clone());

        window.player_box.add(&player.widget);
        window.library_box.add(&library.widget);
        window.search_box.add(&search.widget);

        // Help overlay
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/shortcuts.ui");
        let dialog: gtk::ShortcutsWindow = builder.get_object("shortcuts").unwrap();
        window.widget.set_help_overlay(Some(&dialog));

        let app = Rc::new(Self {
            gtk_app,
            sender,
            receiver,
            window,
            player,
            library,
            search,
        });

        glib::set_application_name(config::NAME);
        glib::set_prgname(Some("shortwave"));
        gtk::Window::set_default_icon_name(config::APP_ID);

        app.setup_gaction();
        app.setup_signals();
        app
    }

    pub fn run(&self, app: Rc<Self>) {
        info!("{}{} ({})", config::NAME_PREFIX, config::NAME, config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        let a = app.clone();
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| a.process_action(action));

        self.gtk_app.run(&[]);
        self.player.shutdown();
    }

    fn setup_gaction(&self) {
        // Quit
        let gtk_app = self.gtk_app.clone();
        self.add_gaction("quit", move |_, _| gtk_app.quit());
        self.gtk_app.set_accels_for_action("app.quit", &["<primary>q"]);

        // About
        let window = self.window.widget.clone();
        self.add_gaction("about", move |_, _| {
            Self::show_about_dialog(window.clone());
        });

        // Search / add stations
        let sender = self.sender.clone();
        self.add_gaction("search", move |_, _| {
            sender.send(Action::ViewShowSearch).unwrap();
        });
        self.gtk_app.set_accels_for_action("app.search", &["<primary>f"]);

        // Import library
        let sender = self.sender.clone();
        self.add_gaction("import-library", move |_, _| {
            sender.send(Action::LibraryImport).unwrap();
        });

        // Export library
        let sender = self.sender.clone();
        self.add_gaction("export-library", move |_, _| {
            sender.send(Action::LibraryExport).unwrap();
        });

        // Sort / Order menu
        let sort_variant = "name".to_variant();
        let sorting_action = gio::SimpleAction::new_stateful("sorting", sort_variant.type_(), &sort_variant);
        self.gtk_app.add_action(&sorting_action);

        let order_variant = "ascending".to_variant();
        let order_action = gio::SimpleAction::new_stateful("order", order_variant.type_(), &order_variant);
        self.gtk_app.add_action(&order_action);

        let sa = sorting_action.clone();
        let oa = order_action.clone();
        let sender = self.sender.clone();
        sorting_action.connect_activate(move |a, b| {
            a.set_state(&b.clone().unwrap());
            Self::sort_action(&sa, &oa, &sender);
        });

        let sa = sorting_action.clone();
        let oa = order_action.clone();
        let sender = self.sender.clone();
        order_action.connect_activate(move |a, b| {
            a.set_state(&b.clone().unwrap());
            Self::sort_action(&sa, &oa, &sender);
        });
    }

    fn sort_action(sorting_action: &gio::SimpleAction, order_action: &gio::SimpleAction, sender: &Sender<Action>) {
        let order_str: String = order_action.get_state().unwrap().get_str().unwrap().to_string();
        let order = match order_str.as_ref() {
            "ascending" => Order::Ascending,
            _ => Order::Descending,
        };

        let sorting_str: String = sorting_action.get_state().unwrap().get_str().unwrap().to_string();
        let sorting = match sorting_str.as_ref() {
            "language" => Sorting::Language,
            "country" => Sorting::Country,
            "state" => Sorting::State,
            "codec" => Sorting::Codec,
            "votes" => Sorting::Votes,
            "bitrate" => Sorting::Bitrate,
            _ => Sorting::Name,
        };

        sender.send(Action::ViewSetSorting(sorting, order)).unwrap();
    }

    fn add_gaction<F>(&self, name: &str, action: F)
    where
        for<'r, 's> F: Fn(&'r gio::SimpleAction, &'s Option<glib::Variant>) + 'static,
    {
        let simple_action = gio::SimpleAction::new(name, None);
        simple_action.connect_activate(action);
        self.gtk_app.add_action(&simple_action);
    }

    fn setup_signals(&self) {
        let window = self.window.widget.clone();
        self.gtk_app.connect_activate(move |app| app.add_window(&window));
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        match action {
            Action::ViewShowSearch => self.window.set_view(View::Search),
            Action::ViewShowLibrary => self.window.set_view(View::Library),
            Action::ViewRaise => self.window.widget.present_with_time((glib::get_monotonic_time() / 1000) as u32),
            Action::ViewShowNotification(text) => self.window.show_notification(text),
            Action::ViewSetSorting(sorting, order) => self.library.set_sorting(sorting, order),
            Action::PlaybackSetStation(station) => {
                self.player.set_station(station.clone());
                self.window.show_sidebar_player(true);
            }
            Action::PlaybackStart => self.player.set_playback(PlaybackState::Playing),
            Action::PlaybackStop => self.player.set_playback(PlaybackState::Stopped),
            Action::LibraryImport => self.import_stations(),
            Action::LibraryExport => self.export_stations(),
            Action::LibraryAddStations(stations) => self.library.add_stations(stations),
            Action::LibraryRemoveStations(stations) => self.library.remove_stations(stations),
            Action::SearchFor(data) => self.search.search_for(data),
        }
        glib::Continue(true)
    }

    fn show_about_dialog(window: gtk::ApplicationWindow) {
        let dialog = gtk::AboutDialog::new();
        dialog.set_program_name(config::NAME);
        dialog.set_logo_icon_name(config::APP_ID);
        dialog.set_comments("A web radio client");
        dialog.set_copyright("© 2019 Felix Häcker");
        dialog.set_license_type(gtk::License::Gpl30);
        dialog.set_version(config::VERSION);
        dialog.set_transient_for(&window);
        dialog.set_modal(true);

        dialog.set_authors(&["Felix Häcker"]);
        dialog.set_artists(&["Tobias Bernard"]);

        dialog.connect_response(|dialog, _| dialog.destroy());
        dialog.show();
    }

    fn import_stations(&self) {
        let import_dialog = gtk::FileChooserNative::new("Select database to import", &self.window.widget, gtk::FileChooserAction::Open, "Import", "Cancel");
        let filter = gtk::FileFilter::new();
        import_dialog.set_filter(&filter);
        filter.add_mime_type("application/json"); // Shortwave library format
        filter.add_mime_type("application/x-sqlite3"); // Old Gradio library format
        filter.add_mime_type("application/vnd.sqlite3"); // Old Gradio library format

        if gtk::ResponseType::from(import_dialog.run()) == gtk::ResponseType::Accept {
            let path = import_dialog.get_file().unwrap().get_path().unwrap();
            debug!("Import path: {:?}", path);
            match Library::read(path) {
                Ok(stations) => {
                    let message = format!("Successfully imported {} stations.", stations.len());
                    self.sender.send(Action::ViewShowNotification(message)).unwrap();

                    self.sender.send(Action::LibraryAddStations(stations));
                }
                Err(error) => {
                    let message = format!("Could not import stations: {}", error.to_string());
                    self.sender.send(Action::ViewShowNotification(message)).unwrap();
                }
            };
        }
        import_dialog.destroy();
    }

    fn export_stations(&self) {
        let export_dialog = gtk::FileChooserNative::new("Export database", &self.window.widget, gtk::FileChooserAction::Save, "Export", "Cancel");
        export_dialog.set_current_name("library.json");
        if gtk::ResponseType::from(export_dialog.run()) == gtk::ResponseType::Accept {
            let path = export_dialog.get_file().unwrap().get_path().unwrap();
            debug!("Export path: {:?}", path);
            let stations = self.library.to_vec();
            let count = stations.len();
            match Library::write(stations, path) {
                Ok(()) => {
                    let message = format!("Successfully exported {} stations.", count);
                    self.sender.send(Action::ViewShowNotification(message)).unwrap();
                }
                Err(error) => {
                    let message = format!("Could not export stations: {}", error.to_string());
                    self.sender.send(Action::ViewShowNotification(message)).unwrap();
                }
            };
        }
        export_dialog.destroy();
    }
}
