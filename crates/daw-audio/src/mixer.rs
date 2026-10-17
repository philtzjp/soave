//! バスチャンネルからハードウェア出力への混合。

use crate::graph::{ChannelKey, Routing};
use crate::source::ToneBank;

/// 経路に従って、バスチャンネルの信号をハードウェア出力に分配します。
#[derive(Debug)]
pub struct Mixer {
    routing: Routing,
    bank: ToneBank,
    /// 経路ごとの、対応する発振器の番号。
    route_tone: Vec<usize>,
    /// 1 フレーム分の、発振器と同じ並びのサンプル。
    frame: Vec<f32>,
}

impl Mixer {
    /// 経路から、既定の音程のトーンで作ります。
    pub fn new(routing: Routing, sample_rate: f32) -> Self {
        let sources = routing.sources();
        Self::with_bank(routing, ToneBank::new(&sources, sample_rate))
    }

    /// 周波数を指定したトーンで作ります。経路に現れるすべてのバスチャンネルの
    /// トーンが要ります。足りないバスチャンネルがあるとパニックします。
    pub fn with_tones(
        routing: Routing,
        tones: &[(ChannelKey, f32)],
        amp: f32,
        sample_rate: f32,
    ) -> Self {
        for key in routing.sources() {
            assert!(
                tones.iter().any(|(k, _)| *k == key),
                "経路元 {key:?} のトーンがありません"
            );
        }
        Self::with_bank(routing, ToneBank::with_tones(tones, amp, sample_rate))
    }

    fn with_bank(routing: Routing, bank: ToneBank) -> Self {
        let keys = bank.keys();
        let route_tone: Vec<usize> = routing
            .routes()
            .iter()
            .map(|route| {
                keys.iter()
                    .position(|key| *key == route.source)
                    .expect("経路元のトーンがありません")
            })
            .collect();
        let frame = vec![0.0; keys.len()];
        Self {
            routing,
            bank,
            route_tone,
            frame,
        }
    }

    /// 全体の長さを指定します。フレーム数です。尻尾にフェードが付きます。
    pub fn set_duration(&mut self, frames: u64) {
        self.bank.set_duration(frames);
    }

    /// サンプルを、ハードウェア出力の並びのインターリーブで書き込みます。
    /// 経路先に既にあるサンプルに、和として加えます。
    pub fn process(&mut self, data: &mut [f32]) {
        let channels = self.routing.device_channels();
        let routes = self.routing.routes();
        for chunk in data.chunks_mut(channels) {
            self.bank.render(&mut self.frame);
            for (route, tone) in routes.iter().zip(&self.route_tone) {
                if let Some(slot) = chunk.get_mut(route.output) {
                    *slot += self.frame[*tone];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Route;

    /// 1 秒分のサンプルレート。
    const RATE: f32 = 48000.0;

    /// バスチャンネルを短く作るための補助。
    fn key(bus: usize, channel: usize) -> ChannelKey {
        ChannelKey::new(bus, channel)
    }

    /// フレーム数分をインターリーブで描く。
    fn render(mixer: &mut Mixer, frames: usize) -> Vec<f32> {
        let channels = mixer.routing.device_channels();
        let mut buf = vec![0.0; frames * channels];
        mixer.process(&mut buf);
        buf
    }

    /// インターリーブの緩衝から、1 チャンネル分を取り出す。
    fn channel(buf: &[f32], ch: usize, channels: usize) -> Vec<f32> {
        buf[ch..].iter().step_by(channels).copied().collect()
    }

    /// ゼロ交差の数から周波数を推定する。
    fn dominant_freq(samples: &[f32], rate: f32) -> f32 {
        let crossings = samples.windows(2).filter(|w| w[0] * w[1] < 0.0).count();
        crossings as f32 / 2.0 / (samples.len() as f32 / rate)
    }

    #[test]
    fn 経路した出力だけに信号があり他は無音になる() {
        let routing = Routing::new(
            [
                Route::new(key(0, 0), 2),
                Route::new(key(0, 1), 5),
                Route::new(key(1, 0), 0),
                Route::new(key(1, 1), 7),
            ],
            8,
        );
        let tones = [
            (key(0, 0), 220.0),
            (key(0, 1), 440.0),
            (key(1, 0), 660.0),
            (key(1, 1), 880.0),
        ];
        let mut mixer = Mixer::with_tones(routing, &tones, 0.2, RATE);
        let buf = render(&mut mixer, RATE as usize);
        for (output, expect) in [(2, 220.0), (5, 440.0), (0, 660.0), (7, 880.0)] {
            let samples = channel(&buf, output, 8);
            let actual = dominant_freq(&samples, RATE);
            assert!(
                (actual - expect).abs() < 2.0,
                "出力 {output} は {expect} Hz のはずが {actual} Hz でした"
            );
            assert!(samples.iter().any(|s| s.abs() > 0.1));
        }
        for output in [1, 3, 4, 6] {
            let samples = channel(&buf, output, 8);
            assert!(
                samples.iter().all(|s| *s == 0.0),
                "出力 {output} は無音のはずでした"
            );
        }
    }

    #[test]
    fn 同じ出力に重ねたときは和になる() {
        let routing = Routing::new([Route::new(key(0, 0), 0), Route::new(key(1, 0), 0)], 2);
        let tones = [(key(0, 0), 440.0), (key(1, 0), 440.0)];
        let mut mixer = Mixer::with_tones(routing, &tones, 0.25, RATE);
        let buf = render(&mut mixer, RATE as usize);
        let peak = channel(&buf, 0, 2)
            .iter()
            .map(|s| s.abs())
            .fold(0.0, f32::max);
        assert!(
            (peak - 0.5).abs() < 0.01,
            "重ねた出力の山は 0.5 のはずが {peak} でした"
        );
        assert!(channel(&buf, 1, 2).iter().all(|s| *s == 0.0));
    }
}
