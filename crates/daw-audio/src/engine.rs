//! デバイスでの再生。

use crate::graph::{ChannelKey, Routing};
use crate::mixer::Mixer;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Error, StreamConfig};

/// 再生の失敗。
#[derive(Debug)]
pub enum PlayError {
    /// デバイス側で失敗しました。
    Device(Error),
    /// 経路のチャンネル数がデバイスの出力チャンネル数を超えています。
    TooManyChannels { wanted: usize, max: usize },
}

impl std::fmt::Display for PlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayError::Device(error) => write!(f, "デバイスでの再生に失敗しました: {error}"),
            PlayError::TooManyChannels { wanted, max } => write!(
                f,
                "経路が {wanted} チャンネルを要しますが、デバイスは {max} チャンネルまでです"
            ),
        }
    }
}

impl std::error::Error for PlayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PlayError::Device(error) => Some(error),
            PlayError::TooManyChannels { .. } => None,
        }
    }
}

impl From<Error> for PlayError {
    fn from(error: Error) -> Self {
        PlayError::Device(error)
    }
}

/// 経路をデバイスで鳴らします。既定の音程のトーンで、seconds 秒鳴らして返ります。
pub fn play(device: &Device, routing: Routing, seconds: f32) -> Result<(), PlayError> {
    let sources = routing.sources();
    let tones: Vec<(ChannelKey, f32)> = sources
        .iter()
        .enumerate()
        .map(|(i, key)| (*key, 220.0 * 2.0_f32.powf(i as f32 / 7.0)))
        .collect();
    play_with_tones(device, routing, &tones, seconds)
}

/// 周波数を指定したトーンで経路を鳴らします。経路に現れるすべてのバスチャンネルの
/// トーンが要ります。足りないバスチャンネルがあるとパニックします。
pub fn play_with_tones(
    device: &Device,
    routing: Routing,
    tones: &[(ChannelKey, f32)],
    seconds: f32,
) -> Result<(), PlayError> {
    let max = crate::device::max_output_channels(device);
    let wanted = routing.device_channels();
    if wanted > max {
        return Err(PlayError::TooManyChannels { wanted, max });
    }
    let default = device.default_output_config()?;
    let config = StreamConfig {
        channels: wanted as u16,
        sample_rate: default.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };
    let rate = config.sample_rate as f32;
    let mut mixer = Mixer::with_tones(routing, tones, 0.2, rate);
    mixer.set_duration((seconds * rate) as u64);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| mixer.process(data),
        |error| eprintln!("ストリームで失敗しました: {error}"),
        None,
    )?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs_f32(seconds));
    drop(stream);
    Ok(())
}
