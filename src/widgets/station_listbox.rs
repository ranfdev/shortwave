use glib::Sender;
use gtk::prelude::*;
use libhandy::{Column, ColumnExt};

use crate::app::Action;
use crate::model::ObjectWrapper;
use crate::model::StationModel;
use crate::widgets::station_row::StationRow;

pub struct StationListBox {
    pub widget: gtk::Box,
    listbox: gtk::ListBox,

    sender: Sender<Action>,
}

impl StationListBox {
    pub fn new(sender: Sender<Action>) -> Self {
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

        Self { widget, listbox, sender }
    }

    pub fn bind_model(&self, model: &StationModel) {
        let sender = self.sender.clone();

        self.listbox.bind_model(&model.model, move |station| {
            let row = StationRow::new(sender.clone(), station.downcast_ref::<ObjectWrapper>().unwrap().deserialize());
            row.widget.upcast::<gtk::Widget>()
        });
    }
}
