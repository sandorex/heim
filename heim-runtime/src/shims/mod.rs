#[cfg(all(feature = "runtime-polyfill", feature = "runtime-async-std"))]
compile_error!("Can't enable multiple runtime shims simultaneously");

cfg_if::cfg_if! {
    if #[cfg(feature = "runtime-polyfill")] {
        mod polyfill;

        pub use self::polyfill::*;
    } else if #[cfg(feature = "runtime-async-std")] {
        mod async_std;

        pub use self::async_std::*;
    }
}
