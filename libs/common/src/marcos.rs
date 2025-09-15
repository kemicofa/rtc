/// Macro that Boxes, Mutexes and Arcs the argument passed.
#[macro_export]
macro_rules! bmarc {
    ($val:expr) => {
        {
        ::std::sync::Arc::new(
            ::tokio::sync::Mutex::new(
                ::std::boxed::Box::new($val),
            )
        )
        }
    };
}

/// Macro that Mutexes and Arcs the argument passed.
#[macro_export]
macro_rules! marc {
    ($val:expr) => {
        {
        ::std::sync::Arc::new(
            ::tokio::sync::Mutex::new(
                $val,
            )
        )
        }
    };
}
