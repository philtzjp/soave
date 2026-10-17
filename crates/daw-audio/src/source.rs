//! テストトーンの音源。

use crate::graph::ChannelKey;

/// フェードの長さ。サンプル数。
const FADE_FRAMES: u64 = 240;

/// 1 つの正弦波の発振器。
#[derive(Clone, Debug)]
struct Oscillator {
    freq: f32,
    phase: f32,
    amp: f32,
}

/// 経路を確かめるためのテストトーンの音源。
///
/// バスチャンネルごとに 1 つの正弦波を持ちます。既定では、バスチャンネルの
/// 並びの順に 220 Hz から五度ずつ上がる音程を割り当てます。知覚で区別できる
/// ようにするためです。頭に短いフェードを付け、全体の長さを指定したときは
/// 尻尾にもフェードを付けます。
#[derive(Clone, Debug)]
pub struct ToneBank {
    sample_rate: f32,
    /// バスチャンネルと発振器の並び。バスチャンネルは鍵として持ちます。
    oscillators: Vec<(ChannelKey, Oscillator)>,
    /// 最初のフレームからの経過。
    frame: u64,
    /// 全体の長さ。None のときは尻尾のフェードを付けません。
    duration: Option<u64>,
}

impl ToneBank {
    /// 経路に現れるバスチャンネルごとに、既定の音程で作ります。
    pub fn new(keys: &[ChannelKey], sample_rate: f32) -> Self {
        let tones: Vec<(ChannelKey, f32)> = keys
            .iter()
            .enumerate()
            .map(|(i, key)| (*key, 220.0 * 2.0_f32.powf(i as f32 / 7.0)))
            .collect();
        Self::with_tones(&tones, 0.2, sample_rate)
    }

    /// 周波数を指定して作ります。
    pub fn with_tones(tones: &[(ChannelKey, f32)], amp: f32, sample_rate: f32) -> Self {
        let oscillators = tones
            .iter()
            .map(|(key, freq)| {
                (
                    *key,
                    Oscillator {
                        freq: *freq,
                        phase: 0.0,
                        amp,
                    },
                )
            })
            .collect();
        Self {
            sample_rate,
            oscillators,
            frame: 0,
            duration: None,
        }
    }

    /// 全体の長さを指定します。フレーム数です。
    pub fn set_duration(&mut self, frames: u64) {
        self.duration = Some(frames);
    }

    /// 発振器のバスチャンネルの並び。
    pub fn keys(&self) -> Vec<ChannelKey> {
        self.oscillators.iter().map(|(key, _)| *key).collect()
    }

    /// 1 フレーム分のサンプルを、発振器と同じ並びで書き込みます。
    pub fn render(&mut self, out: &mut [f32]) {
        let gain = self.gain();
        for ((_, osc), slot) in self.oscillators.iter_mut().zip(out) {
            osc.phase += osc.freq / self.sample_rate;
            if osc.phase >= 1.0 {
                osc.phase -= 1.0;
            }
            *slot = osc.amp * gain * (std::f32::consts::TAU * osc.phase).sin();
        }
        self.frame += 1;
    }

    /// 現在のフレームのフェードを含んだ大きさ。
    fn gain(&self) -> f32 {
        let fade = FADE_FRAMES as f32;
        let mut gain = 1.0;
        if self.frame < FADE_FRAMES {
            gain *= self.frame as f32 / fade;
        }
        if let Some(duration) = self.duration {
            let left = duration.saturating_sub(self.frame);
            if left < FADE_FRAMES {
                gain *= left as f32 / fade;
            }
        }
        gain
    }
}
