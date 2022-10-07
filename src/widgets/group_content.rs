use glib::{ParamFlags, ParamSpec, ParamSpecObject};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use once_cell::sync::{Lazy, OnceCell};

use crate::models::*;
use crate::widgets::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/group-content.ui")]
    pub struct GroupContent {
        #[template_child]
        pub transaction_history: TemplateChild<gtk::ListBox>,

        pub group: OnceCell<Group>,
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
                vec![ParamSpecObject::builder("group", Group::static_type())
                    .flags(ParamFlags::CONSTRUCT | ParamFlags::WRITABLE)
                    .build()]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "group" => {
                    if let Ok(input) = value.get::<Group>() {
                        match self.group.set(input) {
                            Ok(_) => obj.init_transaction_history(),
                            Err(_) => panic!("Group pointer was already set!"),
                        }
                    }
                }
                _ => unimplemented!(),
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
    pub fn new(group: &Group) -> Self {
        glib::Object::new(&[("group", &group)]).expect("Failed to create `GroupContent`.")
    }

    fn init_transaction_history(&self) {
        let group = self.imp().group.get().expect("Group property is not set");

        self.imp()
            .transaction_history
            .bind_model(Some(group.transaction_model()), move |item| {
                let row = item.downcast_ref::<TransactionRow>().unwrap().clone();
                row.upcast::<gtk::Widget>()
            });
    }

    pub fn group(&self) -> &Group {
        self.imp()
            .group
            .get()
            .expect("No Group set for GroupConntent")
    }
}
