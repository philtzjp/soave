//! soave のオーディオ基盤。デバイスの入出力とマルチチャンネルのバス経路を扱う。

pub mod device;
pub mod engine;
pub mod graph;
pub mod mixer;
pub mod source;

pub use graph::{ChannelKey, Route, Routing};
pub use mixer::Mixer;
pub use source::ToneBank;

/// このクレートの版。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn exposes_version() {
        assert!(!VERSION.is_empty());
    }
}
