pub mod v1_10_3 {
    pub const BOOTSTRAP_ICONS_SCSS: &str = include_str!("../res/v1_10_3/scss/bootstrap-icons.scss");
    pub const BOOTSTRAP_ICONS_WOFF: &[u8] =
        include_bytes!("../res/v1_10_3/fonts/bootstrap-icons.woff");
    pub const BOOTSTRAP_ICONS_WOFF2: &[u8] =
        include_bytes!("../res/v1_10_3/fonts/bootstrap-icons.woff2");
}

pub use v1_10_3 as latest;
