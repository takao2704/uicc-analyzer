# uicc-analyzer

Raspberry Pi Pico 2 / Pico 2 W（RP2350）上で動作する、**モデム-UICC 間通信のモニタリング用ファームウェア**です。  
`RST`/`CLK`/`IO` の状態遷移を監視し、USB CDC シリアルへログ出力します。  
あわせてオンボードLEDで状態表示します。

## 目的と非目的

- 目的:
  - モデム-UICC 間の `RST`/`CLK`/`IO` 通信を壊さず受動観測する
  - 通信開始/無信号/ATR待ちなどの状態をすぐ判断できるようにする
- 非目的:
  - 電源クラス（1.8V/3.0V/5V）ネゴシエーションの解析
  - UICC への給電・制御・能動プロービング

## できること（現状）

- `RST` の立ち下がり/立ち上がりを検出してログ出力
- `CLK` の活動を簡易検出してログ出力
- ATR待ち状態への遷移管理（状態機械）
- `IO` サンプル取り込みフック（将来の PIO/DMA 実装用プレースホルダ）
- USB CDC 経由でホストへ時刻付きログ送信
- オンボードLEDで状態表示
  - アイドル: 短い周期点滅
  - ATR待ち: 速い点滅
  - 活動検出中: 点灯
  - 無信号警告: 2連パルス点滅

## ハードウェア接続

RP2350 側の入力ピンは以下です（`src/main.rs`）。

- `GPIO2` : SIM `CLK`
- `GPIO3` : SIM `RST`
- `GPIO4` : SIM `IO`

> 注意:
> - Pico 2 は `GPIO25`、Pico 2 W は `CYW43 GPIO0` を使ってLED制御します。
> - Pico 2 W で CYW43 初期化に失敗した場合、ファームは解析継続のため LED 制御を自動無効化します。
> - SIM観測は受動モニタ前提です。実機接続時は電圧レベル・電源共有・保護回路を必ず確認してください。

## ハードウェア回路設計（追加）

観測用インタフェース回路の推奨構成（レベル変換・ESD 保護・定数例）を
`docs/hardware_circuit_design.md` に追加しました。実機接続前に必ず参照してください。

- 前提レベル変換モジュール: 秋月 `117062`（AE-LLCNV8 / FXMA108）
- 回路図 PDF: `docs/hardware_circuit_diagram.pdf`

## `cyw43` patch運用について

`Cargo.toml` では Pico 2 W 向けの安定動作を優先するため、`cyw43` を一時的に `[patch.crates-io]` で固定しています。

- **なぜ必要か**: Pico 2 W の LED 制御（CYW43 GPIO0）で必要な修正が upstream に未反映のため。
- **いつ外すか**: 対応内容が upstream の正式リリースに取り込まれ、同等の挙動が再現できることを確認できた時点。
- **検証手順の目安**:
  1. `cargo build --release --features pico2w` が通ること。
  2. 起動時に `onboard LED active (CYW43 GPIO0)` が出力されること。
  3. LED self-test（ON/OFF）と通常の状態表示が継続すること。
  4. CYW43 初期化失敗時のフェイルセーフ（監視継続・LED無効化）が維持されること。

追跡用の課題は `docs/issue_tracker.md` の `ISSUE-001` として管理しています。

## ビルド

前提:

- Rust ツールチェーン
- `thumbv8m.main-none-eabihf` ターゲット
- `elf2flash`（RP2350 向け UF2 変換）

```bash
rustup target add thumbv8m.main-none-eabihf
cargo install elf2flash
# Pico 2 (non-W)
~/.cargo/bin/rustup run stable-aarch64-apple-darwin cargo build --release
# Pico 2 W
~/.cargo/bin/rustup run stable-aarch64-apple-darwin cargo build --release --features pico2w
```

## 書き込み（例）

RP2350 を BOOTSEL モードで接続した状態で実行してください。
`.cargo/config.toml` の runner が `tools/deploy_rp2350.sh` を呼び出し、ELF から RP2350 用 UF2 を生成してコピーします。

```bash
# Pico 2 (non-W)
~/.cargo/bin/rustup run stable-aarch64-apple-darwin cargo run --release
# Pico 2 W
~/.cargo/bin/rustup run stable-aarch64-apple-darwin cargo run --release --features pico2w
```

## ログ確認

USB CDC シリアル経由でログを確認できます。

### 1) シリアルモニタを使う

お好みのターミナルツール（`screen`, `minicom`, `picocom` など）で以下のポートを開いてください。

- Linux: `/dev/ttyACM*` や `/dev/ttyUSB*`
- macOS: `/dev/cu.usbmodem*` や `/dev/cu.usbserial*`
- Windows: `COM*`

### 2) 付属スクリプトを使う

`tools/serial_logger.py` を使うとタイムスタンプ付きで表示・保存できます。

```bash
python3 -m pip install pyserial
python3 tools/serial_logger.py --list
python3 tools/serial_logger.py --baud 115200
# ポートを明示する場合（Linux）
python3 tools/serial_logger.py /dev/ttyACM0 --baud 115200
# ポートを明示する場合（macOS）
python3 tools/serial_logger.py /dev/cu.usbmodemXXXX --baud 115200
# 保存する場合
python3 tools/serial_logger.py --save capture.log
```

## 典型ログ例

```text
[0.123 ms] boot
[5.410 ms] RST=LOW
[15.872 ms] RST=HIGH
[16.004 ms] RST released, checking CLK
[16.900 ms] CLK detected
[16.901 ms] waiting for ATR
```

## 実装状況

- ATR のバイト復元・デコード: 未実装
- IO 高速キャプチャ（PIO + DMA）: 未実装
- ETU を考慮したタイミング解析: 未実装

将来的には `io_capture` モジュールを中心に拡張する想定です。

## ライセンス

`Cargo.toml` に記載のとおり、以下のデュアルライセンスです。

- MIT
- Apache-2.0
