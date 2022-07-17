use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use glib::{ParamFlags, ParamSpec, ParamSpecPointer};
use glib::types::Pointee;

use once_cell::sync::{Lazy, OnceCell};

use std::ptr::NonNull;

use crate::models::*;
use crate::widgets::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/group-content.ui")]
    pub struct GroupContent {
        #[template_child]
        pub transaction_history: TemplateChild<gtk::ListBox>,

        pub group_ptr: OnceCell<*const Group>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupContent {
        const NAME: &'static str = "GroupContent";
        type Type = super::GroupContent;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GroupContent {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecPointer::new(
                    "group",
                    "group",
                    "group",
                    ParamFlags::CONSTRUCT | ParamFlags::WRITABLE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "group" => {
                    if let Ok(input) = value.get::<NonNull::<Pointee>>() {
                        let ptr_cast = input.cast::<Group>();
                        match self.group_ptr.set(ptr_cast.as_ptr()) {
                            Ok(_) => obj.init_transaction_history(),
                            Err(_) => panic!("Group pointer was already set!"),
                        }
                    }
                }
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for GroupContent {}
    impl BoxImpl for GroupContent {}
}

glib::wrapper! {
    pub struct GroupContent(ObjectSubclass<imp::GroupContent>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl GroupContent {
    pub fn new(group: *const Group) -> Self {
        let group_ptr = NonNull::new(group as *mut Group)
            .expect("Invalid pointer to group");

        glib::Object::new(&[
            ("group", &group_ptr.cast::<Pointee>().to_value()),
        ]).expect("Failed to create `GroupContent`.")
    }

    fn init_transaction_history(&self) {
        let model = unsafe {
            (*self.imp().group_ptr.get().unwrap())
                .as_ref().unwrap()
                .transaction_model()
        };

        self.imp().transaction_history.bind_model(
            Some(&model),
            glib::clone!(@weak self as parent => @default-panic, move |item| {
                let row = item.downcast_ref::<TransactionRow>().unwrap().clone();
                row.upcast::<gtk::Widget>()
            })
        );
    }
}
