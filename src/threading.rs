//! Wrapper code for co-routines.

#[cfg(not(feature = "async"))]
pub use std::thread::JoinHandle;

#[cfg(feature = "async")]
pub use tokio::task::JoinHandle;

/// Spawns new thread.
#[cfg(not(feature = "async"))]
#[macro_export]
macro_rules! spawn {
    ($rt:expr, $($e:tt)*) => {
        std::thread::spawn(move || {
            $($e)*
        })
    };
}

/// Spawn new thread.
#[cfg(feature = "async")]
#[macro_export]
macro_rules! spawn {
    ($rt:expr, $($e:tt)*) => {
        $rt.spawn(async move {
            $($e)*
        })
    };
}

/// Joins thread.
#[cfg(not(feature = "async"))]
#[macro_export]
macro_rules! join {
    ($rt:expr, $e:expr) => {
        $e.join()
    }
}

/// Joins thread.
#[cfg(feature = "async")]
#[macro_export]
macro_rules! join {
    ($rt:expr, $e:expr) => {
        $rt.block_on($e)
    }
}
