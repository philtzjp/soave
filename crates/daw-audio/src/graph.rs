//! バスと出力の経路。
//!
//! バスはチャンネルの束です。経路は、バスのチャンネルからハードウェア出力への写像です。

/// バスのチャンネルを指す番号の組。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelKey {
    /// バスの番号。
    pub bus: usize,
    /// バスの中のチャンネルの番号。
    pub channel: usize,
}

impl ChannelKey {
    /// バスの番号とチャンネルの番号から作ります。
    pub const fn new(bus: usize, channel: usize) -> Self {
        Self { bus, channel }
    }
}

/// バスのチャンネルからハードウェア出力への経路 1 件。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    /// 経路元のバスチャンネル。
    pub source: ChannelKey,
    /// 経路先のハードウェア出力の番号。
    pub output: usize,
}

impl Route {
    /// バスのチャンネルと出力の番号から作ります。
    pub const fn new(source: ChannelKey, output: usize) -> Self {
        Self { source, output }
    }
}

/// 経路の一覧と、ハードウェア出力のチャンネル数。
///
/// 同じバスチャンネルを複数の出力に出せます。複数のバスチャンネルを同じ出力に
/// 重ねたときは和になります。まったく同じ経路の重複は削ります。
#[derive(Clone, Debug)]
pub struct Routing {
    routes: Vec<Route>,
    device_channels: usize,
}

impl Routing {
    /// 経路の一覧から作ります。出力の番号がチャンネル数を超えているときは
    /// パニックします。
    pub fn new(routes: impl IntoIterator<Item = Route>, device_channels: usize) -> Self {
        let mut routes: Vec<Route> = routes.into_iter().collect();
        routes.sort();
        routes.dedup();
        assert!(
            routes.iter().all(|r| r.output < device_channels),
            "出力の番号がハードウェア出力のチャンネル数 {device_channels} を超えています"
        );
        Self {
            routes,
            device_channels,
        }
    }

    /// 経路の一覧。バス、チャンネル、出力の順に並びます。
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// 経路に現れるバスチャンネルの一覧。バス、チャンネルの順に並び、重複なく
    /// なります。
    pub fn sources(&self) -> Vec<ChannelKey> {
        let mut keys: Vec<ChannelKey> = self.routes.iter().map(|r| r.source).collect();
        keys.sort();
        keys.dedup();
        keys
    }

    /// ハードウェア出力のチャンネル数。
    pub const fn device_channels(&self) -> usize {
        self.device_channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 完全に同じ経路の重複は削られる() {
        let routing = Routing::new(
            [
                Route::new(ChannelKey::new(0, 0), 1),
                Route::new(ChannelKey::new(0, 0), 1),
                Route::new(ChannelKey::new(0, 0), 0),
            ],
            2,
        );
        assert_eq!(routing.routes().len(), 2);
    }

    #[test]
    fn 経路に現れるバスチャンネルは重複なく並ぶ() {
        let routing = Routing::new(
            [
                Route::new(ChannelKey::new(1, 1), 0),
                Route::new(ChannelKey::new(0, 0), 1),
                Route::new(ChannelKey::new(0, 0), 2),
            ],
            4,
        );
        assert_eq!(
            routing.sources(),
            vec![ChannelKey::new(0, 0), ChannelKey::new(1, 1)]
        );
    }

    #[test]
    #[should_panic]
    fn 出力の番号がチャンネル数を超えると失敗する() {
        Routing::new([Route::new(ChannelKey::new(0, 0), 2)], 2);
    }
}
