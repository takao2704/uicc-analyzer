# Issue Tracker

このファイルは、リポジトリに remote が未設定の環境でも追跡できるよう、
フォローアップ課題を一時的に記録するための台帳です。

## ISSUE-001: Remove temporary `cyw43` `[patch.crates-io]`

- **Status**: Open
- **Background**: Pico 2 W の LED 制御安定化のため、現在 `cyw43` は fork の特定コミットへ pin されています。
- **Goal**: upstream の正式リリースへ移行し、`[patch.crates-io]` を削除する。
- **Exit Criteria**:
  1. `Cargo.toml` の `cyw43` patch が不要であること。
  2. `--features pico2w` ビルドが通ること。
  3. LED self-test と通常状態表示が維持されること。
  4. 初期化失敗時のフェイルセーフ挙動が維持されること。
