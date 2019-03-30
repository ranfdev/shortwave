use glib::Sender;
use gtk::prelude::*;

use crate::app::Action;
use crate::model::ObjectWrapper;
use crate::model::SongModel;
use crate::widgets::song_row::SongRow;

pub struct SongListBox {
    pub widget: gtk::Box,
    listbox: gtk::ListBox,

    sender: Sender<Action>,
}

impl SongListBox {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/de/haeckerfelix/Shortwave/gtk/song_listbox.ui");
        let widget: gtk::Box = builder.get_object("song_listbox").unwrap();
        let listbox: gtk::ListBox = builder.get_object("listbox").unwrap();

        Self { widget, listbox, sender }
    }

    pub fn bind_model(&self, model: &SongModel) {
        let sender = self.sender.clone();

        self.listbox.bind_model(&model.model, move |song| {
            let row = SongRow::new(sender.clone(), song.downcast_ref::<ObjectWrapper>().unwrap().deserialize());
            row.widget.upcast::<gtk::Widget>()
        });
    }
}
