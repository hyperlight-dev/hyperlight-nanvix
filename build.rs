fn main() {
    #[cfg(feature = "napi")]
    {
        use napi_build::setup;
        setup();
    }
}