/// Run expression with `Result<(), Error>` return type and panic if it returns error.
#[macro_export]
macro_rules! force {
    ( $x:expr ) => {
        match $x {
            Ok(()) => { }
            Err(error) => { panic!("{}", error); }
        }
    };
}
