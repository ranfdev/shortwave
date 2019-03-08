use glib::Sender;
use gtk::prelude::*;
use libhandy::{Column, ColumnExt};

use crate::app::Action;
use crate::station_model::StationModel;
use crate::station_object::StationObject;
use crate::widgets::station_row::{ContentType, StationRow};

pub struct StationListBox {
    pub widget: gtk::Box,
    listbox: gtk::ListBox,
    content_type: ContentType,

    sender: Sender<Action>,
}

impl StationListBox {
    pub fn new(sender: Sender<Action>, content_type: ContentType) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_listbox.ui");
        let widget: gtk::Box = builder.get_object("station_listbox").unwrap();
        let listbox: gtk::ListBox = builder.get_object("listbox").unwrap();

        // Setup HdyColumn
        let column = Column::new();
        column.set_maximum_width(700);
        widget.add(&column);
        let column = column.upcast::<gtk::Widget>(); // See https://gitlab.gnome.org/World/podcasts/blob/master/podcasts-gtk/src/widgets/home_view.rs#L64
        let column = column.downcast::<gtk::Container>().unwrap();
        column.show();
        column.add(&listbox);

        Self {
            widget,
            listbox,
            content_type,
            sender,
        }
    }

    pub fn bind_model(&self, model: &StationModel) {
        let sender = self.sender.clone();
        let content_type = self.content_type.clone();

        self.listbox.bind_model(&model.model, move |station| {
            let row = StationRow::new(sender.clone(), station.downcast_ref::<StationObject>().unwrap().to_station(), content_type.clone());
            row.widget.upcast::<gtk::Widget>()
        });
    }
}
