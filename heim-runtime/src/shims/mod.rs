cfg_if::cfg_if! {
    if #[cfg(feature = "runtime-polyfill")] {
        mod sync;

        pub use self::sync::*;
    } else if #[cfg(feature = "runtime-tokio")] {
        mod tokio;

        pub use self::tokio::*;
    } else {
        compile_error!("No shim runtime was selected");
    }
}
