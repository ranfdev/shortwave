// StationObject is a GObject subclass, which we need to carry the rustio::Station struct.
// With this we can use gtk::ListBox bind_model() properly.
//
// For more details, you should look at this gtk-rs example:
// https://github.com/gtk-rs/examples/blob/master/src/bin/listbox_model.rs

use super::*;
use gtk::prelude::*;
use rustio::Station;

use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

mod imp {
    use super::*;
    use std::cell::RefCell;

    pub struct StationObject {
        data: RefCell<Option<String>>,
    }

    static PROPERTIES: [subclass::Property; 1] = [subclass::Property("data", |name| {
        glib::ParamSpec::string(
            name,
            "Data",
            "Data",
            None, // Default value
            glib::ParamFlags::READWRITE,
        )
    })];

    impl ObjectSubclass for StationObject {
        const NAME: &'static str = "StationObject";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;

        glib_object_subclass!();

        fn class_init(klass: &mut Self::Class) {
            klass.install_properties(&PROPERTIES);
        }

        fn new() -> Self {
            Self { data: RefCell::new(None) }
        }
    }

    impl ObjectImpl for StationObject {
        glib_object_impl!();

        fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("data", ..) => {
                    let data = value.get();
                    self.data.replace(data);
                }
                _ => unimplemented!(),
            }
        }

        fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
            let prop = &PROPERTIES[id];

            match *prop {
                subclass::Property("data", ..) => Ok(self.data.borrow().to_value()),
                _ => unimplemented!(),
            }
        }
    }
}

glib_wrapper! {
    pub struct StationObject(Object<subclass::simple::InstanceStruct<imp::StationObject>, subclass::simple::ClassStruct<imp::StationObject>, StationObjectClass>);

    match fn {
        get_type => || imp::StationObject::get_type().to_glib(),
    }
}

impl StationObject {
    pub fn new(station: Station) -> StationObject {
        glib::Object::new(Self::static_type(), &[("data", &serde_json::to_string(&station).unwrap())])
            .unwrap()
            .downcast()
            .unwrap()
    }

    pub fn to_station(&self) -> Station {
        let data = self.get_property("data").unwrap().get::<String>().unwrap();
        serde_json::from_str(&data).unwrap()
    }
}
