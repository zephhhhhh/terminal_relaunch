#[allow(unused_macros)]
macro_rules! info {
    ($($arg:tt)+) => {
        #[cfg(feature = "logging")]
        {
            log::info!($($arg)+);
        }
    };
}

#[allow(unused_macros)]
macro_rules! warning {
    ($($arg:tt)+) => {
        #[cfg(feature = "logging")]
        {
            log::warn!($($arg)+);
        }
    };
}

#[allow(unused_macros)]
macro_rules! error {
    ($($arg:tt)+) => {
        #[cfg(feature = "logging")]
        {
            log::error!($($arg)+);
        }
    };
}

#[allow(unused_macros)]
macro_rules! debug {
    ($($arg:tt)+) => {
        #[cfg(feature = "logging")]
        {
            log::debug!($($arg)+);
        }
    };
}

#[allow(unused_macros)]
macro_rules! trace {
    ($($arg:tt)+) => {
        #[cfg(feature = "logging")]
        {
            log::trace!($($arg)+);
        }
    };
}

#[allow(unused_imports)]
pub(crate) use {debug, error, info, trace, warning};
