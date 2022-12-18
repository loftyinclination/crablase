pub mod game;

mod routes {
    macro_rules! asset {
        ($path:expr) => {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
        };
    }
    pub(crate) use asset;
}
