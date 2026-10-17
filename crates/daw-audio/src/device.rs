//! 出力デバイスの列挙。

use cpal::traits::{DeviceTrait, HostTrait};

/// 出力デバイスの情報。
#[derive(Clone, Debug)]
pub struct OutputDeviceInfo {
    /// デバイスの名前。
    pub name: String,
    /// 出力チャンネルの最大数。
    pub max_channels: usize,
    /// 既定のサンプルレート。取れないときは 0 です。
    pub default_sample_rate: u32,
}

/// 既定のホストの出力デバイスを一覧します。並びは環境が返すままです。
pub fn output_devices() -> Vec<OutputDeviceInfo> {
    cpal::default_host()
        .output_devices()
        .map(|devices| devices.map(|device| info_of(&device)).collect())
        .unwrap_or_default()
}

/// 番号で出力デバイスを取ります。番号は output_devices の並びです。
pub fn output_device(index: usize) -> Option<cpal::Device> {
    cpal::default_host().output_devices().ok()?.nth(index)
}

/// デバイスが出せる出力チャンネルの数。対応する設定が一つもないときは 0 です。
pub fn max_output_channels(device: &cpal::Device) -> usize {
    device
        .supported_output_configs()
        .map(|ranges| {
            ranges
                .map(|range| range.channels() as usize)
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

fn info_of(device: &cpal::Device) -> OutputDeviceInfo {
    let name = device
        .description()
        .map(|description| description.name().to_string())
        .unwrap_or_else(|_| "名前の取れないデバイス".to_string());
    let max_channels = max_output_channels(device);
    let default_sample_rate = device
        .default_output_config()
        .map(|config| config.sample_rate())
        .unwrap_or(0);
    OutputDeviceInfo {
        name,
        max_channels,
        default_sample_rate,
    }
}
