//! デバイスの一覧と経路を指定した再生を確かめる。

use daw_audio::device::{max_output_channels, output_device, output_devices};
use daw_audio::graph::{ChannelKey, Route, Routing};
use std::process::ExitCode;

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("devices") => devices(),
        Some("play") => play(),
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("soave-probe devices                             出力デバイスの一覧");
    eprintln!("soave-probe play [秒] [--device N] [--route bus.チャンネル:出力] ...");
    eprintln!(
        "                                                例: soave-probe play 2 --device 12 --route 0.0:0 --route 0.1:7"
    );
    ExitCode::from(2)
}

fn devices() -> ExitCode {
    for (index, info) in output_devices().iter().enumerate() {
        println!(
            "{index}: {} ({}ch, {} Hz)",
            info.name, info.max_channels, info.default_sample_rate
        );
    }
    ExitCode::SUCCESS
}

fn play() -> ExitCode {
    let mut seconds = 3.0_f32;
    let mut device_index = 0_usize;
    let mut routes: Vec<Route> = vec![];
    let args: Vec<String> = std::env::args().skip(2).collect();
    let mut flag = false;
    for arg in &args {
        if let Some(rest) = arg.strip_prefix("--device ") {
            match rest.parse() {
                Ok(n) => device_index = n,
                Err(_) => return fail(&format!("--device は番号です: {rest}")),
            }
        } else if let Some(rest) = arg.strip_prefix("--route ") {
            match parse_route(rest) {
                Some(route) => routes.push(route),
                None => return fail(&format!("--route は bus.チャンネル:出力 の形です: {rest}")),
            }
        } else if let Some(rest) = arg.strip_prefix("--") {
            let _ = rest;
            flag = true;
        } else if !flag {
            match arg.parse() {
                Ok(value) => seconds = value,
                Err(_) => return fail(&format!("秒は数値です: {arg}")),
            }
        }
    }
    if routes.is_empty() {
        routes.push(Route::new(ChannelKey::new(0, 0), 0));
        routes.push(Route::new(ChannelKey::new(0, 1), 1));
    }
    let Some(device) = output_device(device_index) else {
        for (index, info) in output_devices().iter().enumerate() {
            eprintln!("{index}: {} ({}ch)", info.name, info.max_channels);
        }
        return fail(&format!("デバイスが見つかりません: {device_index}"));
    };
    let channels = max_output_channels(&device);
    if channels == 0 {
        return fail(&format!(
            "デバイス {device_index} は出力チャンネルを持てません"
        ));
    }
    let routing = Routing::new(routes, channels);
    println!("デバイス {device_index} で {seconds} 秒、{channels} チャンネルに経路します");
    match daw_audio::engine::play(&device, routing, seconds) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&format!("再生に失敗しました: {error}")),
    }
}

fn parse_route(spec: &str) -> Option<Route> {
    let (source, output) = spec.split_once(':')?;
    let (bus, channel) = source.split_once('.')?;
    Some(Route::new(
        ChannelKey::new(bus.parse().ok()?, channel.parse().ok()?),
        output.parse().ok()?,
    ))
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}
