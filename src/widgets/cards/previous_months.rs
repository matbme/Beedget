use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Debug, Default)]
    #[template(resource = "/com/github/matbme/beedget/ui/cards/previous-months.ui")]
    pub struct PreviousMonths {

    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreviousMonths {
        const NAME: &'static str = "PreviousMonths";
        type Type = super::PreviousMonths;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            // Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreviousMonths {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for PreviousMonths {}
    impl WindowImpl for PreviousMonths {}
    impl BoxImpl for PreviousMonths {}
}

glib::wrapper! {
    pub struct PreviousMonths(ObjectSubclass<imp::PreviousMonths>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PreviousMonths {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `DateTimePicker`.")
    }
}
