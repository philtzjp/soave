# soave

soave は、Rust で作るヘッドレス DAW のコアエンジンです。UI を持たず、音楽記法をネイティブのセッション形式にして、MCP で操作します。プロトコルを公開して、誰でも自分の UI を組めるようにします。

> soave is a headless DAW engine in Rust. It ships no UI of its own; sessions are written in a music-notation-native format and driven through MCP, so anyone can build their own front end on top of the open protocol.

## 方針

- マルチチャンネルを最初から扱います。複数チャンネルのオーディオを複数チャンネルのバスに通して、複数のハードウェア出力に出します
- プラグインのホストは VST3 に絞ります
- トランスポートとスケジューラはサンプル精度で動かします
- セッションの形式は音楽記法をネイティブにします
- ライセンスは FSL-1.1-ALv2 で、2 年後に Apache-2.0 に変わります

## 構成

```text
crates/
├── daw-audio     デバイスとバスとマルチチャンネルの経路
├── daw-notation  音楽記法のパーサとライタ、今後
├── daw-core      ステート管理、今後
├── daw-host      VST3 プラグインホスト、今後
└── daw-mcp       MCP サーバー、今後
```

## 開発

```sh
pnpm install
cargo build
cargo test
```

開発の手順は [AGENTS.md](AGENTS.md) と [philtzjp/startingpoint](https://github.com/philtzjp/startingpoint) の START.md に従います。

## ライセンス

[FSL-1.1-ALv2](LICENSE) です。2 年後に Apache-2.0 に変わります。
