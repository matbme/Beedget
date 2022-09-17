/// Run expression with `Result<(), Error>` return type and panic if it returns error.
#[macro_export]
macro_rules! force {
    ( $x:expr ) => {
        match $x {
            Ok(()) => {}
            Err(error) => {
                panic!("{:?}", error);
            }
        }
    };
}

/// Get application object from reference downcast to specified type
///
/// **Ex:**
/// ```
/// let application = application!(self @as crate::BeedgetWindow);
/// ```
#[macro_export]
macro_rules! application {
    ($ref:ident @as $downcast:path) => {{
        let root = $ref.root().expect("Widget does not contain a root");

        let window = root
            .downcast_ref::<gtk::Window>()
            .expect("Widget's root is not a GTK Window");

        let application = window.application().unwrap_or_else(|| {
            window
                .transient_for()
                .expect("Window is not transient for anything.")
                .application()
                .expect("No application set for transient window")
        });

        application.downcast::<$downcast>().expect(&format!(
            "Could not downcast Application to {}",
            stringify!($downcast)
        ))
    }};
}
