#[cfg(feature = "electrs_0_5_0")]
const VERSION: &str = "v0.5.0";

#[cfg(feature = "electrs_0_5_1")]
const VERSION: &str = "v0.5.1";

#[cfg(not(any(feature = "electrs_0_5_0", feature = "electrs_0_5_1",)))]
const VERSION: &str = "NA";

pub const HAS_FEATURE: bool = cfg!(any(feature = "electrs_0_5_0", feature = "electrs_0_5_1",));

pub fn electrs_name() -> String {
    format!("esplora-tapyrus-{}-x86_64-unknown-linux-gnu", VERSION)
}
