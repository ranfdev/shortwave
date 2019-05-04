use glib::Sender;
use gtk::prelude::*;
use libhandy::{Column, ColumnExt};

use crate::app::Action;
use crate::model::ObjectWrapper;
use crate::model::StationModel;
use crate::widgets::station_row::StationRow;

pub struct StationFlowBox {
    pub widget: gtk::Box,
    flowbox: gtk::FlowBox,

    sender: Sender<Action>,
}

impl StationFlowBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/station_flowbox.ui");
        let widget: gtk::Box = builder.get_object("station_flowbox").unwrap();
        let flowbox: gtk::FlowBox = builder.get_object("flowbox").unwrap();

        // Setup HdyColumn
        let column = Column::new();
        column.set_maximum_width(1600);
        widget.add(&column);
        let column = column.upcast::<gtk::Widget>(); // See https://gitlab.gnome.org/World/podcasts/blob/master/podcasts-gtk/src/widgets/home_view.rs#L64
        let column = column.downcast::<gtk::Container>().unwrap();
        column.show();
        column.add(&flowbox);

        // Set automatically flowbox colums
        let fb = flowbox.clone();
        flowbox.connect_size_allocate(move |_, alloc| {
            if alloc.width > 1000 {
                fb.set_min_children_per_line(3);
                fb.set_max_children_per_line(3);
            } else if alloc.width > 650 {
                fb.set_min_children_per_line(2);
                fb.set_max_children_per_line(2);
            } else {
                fb.set_min_children_per_line(1);
                fb.set_max_children_per_line(1);
            }
        });

        Self { widget, flowbox, sender }
    }

    pub fn bind_model(&self, model: &StationModel) {
        let sender = self.sender.clone();

        self.flowbox.bind_model(&model.model, move |station| {
            let row = StationRow::new(sender.clone(), station.downcast_ref::<ObjectWrapper>().unwrap().deserialize());
            row.widget.upcast::<gtk::Widget>()
        });
    }
}
