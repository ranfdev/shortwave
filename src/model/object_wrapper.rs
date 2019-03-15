// ObjectWrapper is a GObject subclass, which we need to carry the rustio::Station/song::Song struct.
// With this we can use gtk::ListBox bind_model() properly.
//
// For more details, you should look at this gtk-rs example:
// https://github.com/gtk-rs/examples/blob/master/src/bin/listbox_model.rs

use gtk::prelude::*;
use serde::de::DeserializeOwned;

use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

mod imp {
    use super::*;
    use std::cell::RefCell;

    pub struct ObjectWrapper {
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

    impl ObjectSubclass for ObjectWrapper {
        const NAME: &'static str = "ObjectWrapper";
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

    impl ObjectImpl for ObjectWrapper {
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
    pub struct ObjectWrapper(Object<subclass::simple::InstanceStruct<imp::ObjectWrapper>, subclass::simple::ClassStruct<imp::ObjectWrapper>, ObjectWrapperClass>);

    match fn {
        get_type => || imp::ObjectWrapper::get_type().to_glib(),
    }
}

impl ObjectWrapper {
    pub fn new<O>(object: O) -> ObjectWrapper
    where
        O: serde::ser::Serialize,
    {
        glib::Object::new(Self::static_type(), &[("data", &serde_json::to_string(&object).unwrap())])
            .unwrap()
            .downcast()
            .unwrap()
    }

    pub fn deserialize<O>(&self) -> O
    where
        O: DeserializeOwned,
    {
        let data = self.get_property("data").unwrap().get::<String>().unwrap();
        serde_json::from_str(&data).unwrap()
    }
}
