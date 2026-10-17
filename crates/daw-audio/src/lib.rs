//! soave のオーディオ基盤。デバイスの入出力とマルチチャンネルのバス経路を扱う。
//!
//! このクレートは開発の最初の段階で、中身はこれから作る。

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
