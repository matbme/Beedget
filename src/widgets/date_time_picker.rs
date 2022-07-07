use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use glib::{ParamFlags, ParamSpec, ParamSpecGType};

use once_cell::sync::Lazy;
use derivative::*;

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Derivative, CompositeTemplate)]
    #[derivative(Debug, Default)]
    #[template(resource = "/com/github/matbme/beedget/ui/date-time-picker.ui")]
    pub struct DateTimePicker {
        #[template_child]
        pub calendar_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub calendar: TemplateChild<gtk::Calendar>,

        #[template_child]
        pub date_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub hour_increase: TemplateChild<gtk::Button>,
        #[template_child]
        pub hour_value: TemplateChild<gtk::Entry>,
        #[template_child]
        pub hour_decrease: TemplateChild<gtk::Button>,

        #[template_child]
        pub minute_increase: TemplateChild<gtk::Button>,
        #[template_child]
        pub minute_value: TemplateChild<gtk::Entry>,
        #[template_child]
        pub minute_decrease: TemplateChild<gtk::Button>,

        #[derivative(Default(value="RefCell::new(glib::DateTime::now_local().unwrap())"))]
        pub selected_date: RefCell<glib::DateTime>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DateTimePicker {
        const NAME: &'static str = "DateTimePicker";
        type Type = super::DateTimePicker;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DateTimePicker {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecGType::new(
                    "selected-date",
                    "selected-date",
                    "selected-date",
                    glib::DateTime::static_type(),
                    ParamFlags::READABLE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "selected-date" => {
                    if let Ok(input) = value.get() {
                        self.selected_date.replace(input);
                    }
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-date" => self.selected_date.borrow().to_value(),
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Initialize entries
            obj.update_date_entry_text();
            obj.update_hour_entry();
            obj.update_minute_entry();

            //Connect entries' changes
            obj.connect_date_entry_changes();
            obj.connect_hour_entry_changes();
            obj.connect_minute_entry_changes();
        }
    }
    impl WidgetImpl for DateTimePicker {}
    impl WindowImpl for DateTimePicker {}
    impl BoxImpl for DateTimePicker {}
}

glib::wrapper! {
    pub struct DateTimePicker(ObjectSubclass<imp::DateTimePicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

#[gtk::template_callbacks]
impl DateTimePicker {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create `DateTimePicker`.")
    }

    #[template_callback]
    fn calendar_popdown(&self) {
        self.imp().calendar_popover.popup();
    }

    #[template_callback]
    fn increase_hour(&self) {
        let new_date = self.imp().selected_date.borrow().add_hours(1);

        match new_date {
            Ok(date) => _ = self.imp().selected_date.replace(date),
            Err(_) => { }
        }

        self.update_hour_entry();
    }

    #[template_callback]
    fn decrease_hour(&self) {
        let new_date = self.imp().selected_date.borrow().add_hours(-1);

        match new_date {
            Ok(date) => _ = self.imp().selected_date.replace(date),
            Err(_) => { }
        }

        self.update_hour_entry();
    }

    #[template_callback]
    fn increase_minute(&self) {
        let new_date = self.imp().selected_date.borrow().add_minutes(1);

        match new_date {
            Ok(date) => _ = self.imp().selected_date.replace(date),
            Err(_) => { }
        }

        self.update_minute_entry();
    }

    #[template_callback]
    fn decrease_minute(&self) {
        let new_date = self.imp().selected_date.borrow().add_minutes(-1);

        match new_date {
            Ok(date) => _ = self.imp().selected_date.replace(date),
            Err(_) => { }
        }

        self.update_minute_entry();
    }

    #[template_callback]
    fn set_day_selected(&self, calendar: gtk::Calendar) {
        let new_date: glib::DateTime;
        {
            let current_date = self.imp().selected_date.borrow();

            new_date = glib::DateTime::new(
                &current_date.timezone(),
                calendar.year(),
                calendar.month(),
                calendar.day(),
                current_date.hour(),
                current_date.minute(),
                current_date.seconds()
            ).unwrap();
        }

        self.imp().selected_date.replace(new_date);
        self.update_date_entry_text();
    }

    /// Update date select entry based on `selected_date`
    fn update_date_entry_text(&self) {
        let formatted = if let Ok(form) = self.imp().selected_date.borrow().format("%x") {
            form
        } else {
            self.imp().selected_date.borrow().format("F").unwrap()
        };

        self.imp().date_entry.set_text(&formatted);
    }

    /// Update hour entry based on `selected_date`
    fn update_hour_entry(&self) {
        let hour = self.imp().selected_date.borrow().hour();
        self.imp().hour_value.set_text(&format!("{}", hour));
    }

    /// Update minute entry based on `selected_date`
    fn update_minute_entry(&self) {
        let minute = self.imp().selected_date.borrow().minute();
        self.imp().minute_value.set_text(&format!("{}", minute));
    }

    /// Update date and calendar popover when date entry changes
    fn connect_date_entry_changes(&self) {
        self.imp().date_entry.connect_text_notify(glib::clone!(@weak self as parent => move |_| {
            if let Some(date) = parent.parse_date() {
                parent.imp().calendar.set_year(date.year());
                parent.imp().calendar.set_month(date.month());
                parent.imp().calendar.set_day(date.day_of_month());

                let new_date: glib::DateTime = {
                    let current_date = parent.imp().selected_date.borrow();
                    glib::DateTime::new(
                        &current_date.timezone(),
                        date.year(),
                        date.month(),
                        date.day_of_month(),
                        current_date.hour(),
                        current_date.minute(),
                        current_date.seconds()
                    ).unwrap()
                };

                parent.imp().selected_date.replace(new_date);
                parent.imp().date_entry.remove_css_class("error");
            } else {
                parent.imp().date_entry.add_css_class("error");
            }
        }));
    }

    /// Update selected date when hour entry changes
    fn connect_hour_entry_changes(&self) {
        self.imp().hour_value.connect_text_notify(glib::clone!(@weak self as parent => move |_| {
            if let Ok(value) = parent.imp().hour_value.text().as_str().parse::<i32>() {
                if let Ok(new_date) = {
                    let current_date = parent.imp().selected_date.borrow();
                    glib::DateTime::new(
                        &current_date.timezone(),
                        current_date.year(),
                        current_date.month(),
                        current_date.day_of_month(),
                        value,
                        current_date.minute(),
                        current_date.seconds()
                    )
                } {
                    parent.imp().selected_date.replace(new_date);
                    parent.imp().hour_value.remove_css_class("error");
                }
                else {
                    parent.imp().hour_value.add_css_class("error")
                }
            }
        }));
    }

    /// Update selected date when minute entry changes
    fn connect_minute_entry_changes(&self) {
        self.imp().minute_value.connect_text_notify(glib::clone!(@weak self as parent => move |_| {
            if let Ok(value) = parent.imp().minute_value.text().as_str().parse::<i32>() {
                if let Ok(new_date) = {
                    let current_date = parent.imp().selected_date.borrow();
                    glib::DateTime::new(
                        &current_date.timezone(),
                        current_date.year(),
                        current_date.month(),
                        current_date.day_of_month(),
                        current_date.hour(),
                        value,
                        current_date.seconds()
                    )
                } {
                    parent.imp().selected_date.replace(new_date);
                    parent.imp().minute_value.remove_css_class("error");
                }
                else {
                    parent.imp().minute_value.add_css_class("error")
                }
            }
        }));
    }

    /// `GLib::Date set_parse` wasn't warking so I made my own unsafe binding to
    /// g_date_set_parse out of spite
    fn parse_date(&self) -> Option<glib::DateTime> {
        let mut parsed_date = glib::ffi::GDate {
            julian_days: 100,
            flags_dmy: 0
        };

        unsafe {
            glib::ffi::g_date_clear(&mut parsed_date, 1);

            let entry_text = self.imp().date_entry.text().into_raw();
            glib::ffi::g_date_set_parse(&mut parsed_date, entry_text);

            if glib::ffi::g_date_valid(&parsed_date) != 0 {
                Some(glib::DateTime::from_local(
                    glib::ffi::g_date_get_year(&parsed_date).into(),
                    glib::ffi::g_date_get_month(&parsed_date).into(),
                    glib::ffi::g_date_get_day(&parsed_date).into(),
                    0, 0, 0.0
                ).unwrap())
            } else {
                None
            }
        }
    }
}
