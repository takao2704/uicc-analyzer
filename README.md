# uicc-analyzer

Raspberry Pi Pico（RP2040）上で動作する、**UICC/SIM 信号の観測用ファームウェア**です。  
現状は最小構成として、`RST`/`CLK`/`IO` の状態遷移を監視し、USB CDC シリアルへログ出力します。

## できること（現状）

- `RST` の立ち下がり/立ち上がりを検出してログ出力
- `CLK` の活動を簡易検出してログ出力
- ATR待ち状態への遷移管理（状態機械）
- `IO` サンプル取り込みフック（将来の PIO/DMA 実装用プレースホルダ）
- USB CDC 経由でホストへ時刻付きログ送信

## ハードウェア接続

RP2040 側の入力ピンは以下です（`src/main.rs`）。

- `GPIO2` : SIM `CLK`
- `GPIO3` : SIM `RST`
- `GPIO4` : SIM `IO`

> 注意: 本プロジェクトは「観測」を目的とした実装です。実際のカード/端末に接続する場合は、必ず電圧レベル・電源共有・保護回路を確認してください。

## ハードウェア回路設計（追加）

観測用インタフェース回路の推奨構成（レベル変換・ESD 保護・定数例）を
`docs/hardware_circuit_design.md` に追加しました。実機接続前に必ず参照してください。

## ビルド

前提:

- Rust ツールチェーン
- `thumbv6m-none-eabi` ターゲット
- UF2 書き込み環境（例: `elf2uf2-rs` や probe-rs）

```bash
rustup target add thumbv6m-none-eabi
cargo build --release --target thumbv6m-none-eabi
```

## 書き込み（例）

使用する環境に合わせて実施してください。例えば UF2 で書き込む場合:

1. 生成された ELF から UF2 を作成
2. BOOTSEL モードの Pico へコピー

（本リポジトリでは書き込み手順自体は固定していません）

## ログ確認

USB CDC シリアル経由でログを確認できます。

### 1) シリアルモニタを使う

お好みのターミナルツール（`screen`, `minicom`, `picocom` など）で `/dev/ttyACM*` を開いてください。

### 2) 付属スクリプトを使う

`tools/serial_logger.py` を使うとタイムスタンプ付きで表示・保存できます。

```bash
python3 -m pip install pyserial
python3 tools/serial_logger.py /dev/ttyACM0 --baud 115200
# 保存する場合
python3 tools/serial_logger.py /dev/ttyACM0 --save capture.log
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
