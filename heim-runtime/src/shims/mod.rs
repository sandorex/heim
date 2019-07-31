cfg_if::cfg_if! {
    if #[cfg(feature = "reactor-polyfill")] {
        mod sync;

        pub use self::sync::*;
    } else if #[cfg(feature = "reactor-tokio")] {
        mod tokio;

        pub use self::tokio::*;
    } else {
        compile_error!("No shim reactor was selected");
    }
}
