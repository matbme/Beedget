use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/empty-dialog.ui")]
    pub struct EmptyDialog {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,

        #[template_child]
        pub heading: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EmptyDialog {
        const NAME: &'static str = "EmptyDialog";
        type Type = super::EmptyDialog;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EmptyDialog {}
    impl WidgetImpl for EmptyDialog {}
    impl BoxImpl for EmptyDialog {}
}

glib::wrapper! {
    pub struct EmptyDialog(ObjectSubclass<imp::EmptyDialog>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl EmptyDialog {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `EmptyDialog`.")
    }
}
