use derive_yew_bootstrap_icons::YewBootstrapIcons;

#[derive(YewBootstrapIcons)]
#[yew_bootstrap_icons(
    mod_name = "v1_10_3",
    json_path = "./res/v1_10_3/bootstrap-icons.json",
    prefix = "Bi",
    always_add_prefix = false,
    default = "Question"
)]
#[allow(dead_code)]
struct GenV1_10_3 {}

pub use v1_10_3 as latest;
