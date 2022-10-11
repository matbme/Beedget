use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Debug, Default)]
    #[template(resource = "/com/github/matbme/beedget/ui/cards/monthly-expenses.ui")]
    pub struct MonthlyExpenses {

    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthlyExpenses {
        const NAME: &'static str = "MonthlyExpenses";
        type Type = super::MonthlyExpenses;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            // Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MonthlyExpenses {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for MonthlyExpenses {}
    impl WindowImpl for MonthlyExpenses {}
    impl BoxImpl for MonthlyExpenses {}
}

glib::wrapper! {
    pub struct MonthlyExpenses(ObjectSubclass<imp::MonthlyExpenses>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MonthlyExpenses {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `DateTimePicker`.")
    }
}
