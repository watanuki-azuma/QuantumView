# QuantumView

Chromiumを使わずにHTML/CSSのデスクトップUIを描画できるか、ServoとBlitzで確かめる最初の試作です。採用エンジンはまだ決めていません。

同じ大きさのウィンドウに、日本語メモ、文字数、クリアボタン、100件のカードを表示します。入力、IME、クリップボード、スクロール、リサイズと実際のリソース使用量を比較します。

| 実行ファイル | 描画 | UIロジック | 用途 |
| --- | --- | --- | --- |
| `quantumview-servo` | Servo 0.7.0 / WebRender | ローカルHTML/CSS/JS | 候補 |
| `quantumview-blitz` | Blitz / Vello（Dioxus Native 0.7.10） | Rust / Dioxus | 候補 |
| `quantumview-webview2` | Windows WebView2（Chromium） | Servoと同じHTML/CSS/JS | 比較基準 |

3つは独立した小さなCargoプロジェクトです。エンジン固有の入力処理を比較するため、それぞれ直接起動します。依存関係の解決を分離することで、BlitzやWebView2だけのビルドでServoを取得する必要もありません。共通にするのは画面のCSSと比較手順だけです。

```text
probes/servo/       Servoのウィンドウ・描画・入力
probes/blitz/       Dioxus Nativeの画面
probes/webview2/    wryによるWindows比較版
ui/                共通CSS、Servo/WebView2用HTMLとJS
scripts/measure.ps1 プロセス群のメモリとCPUをCSVに記録
docs/              比較手順、検証結果
measurements/      実測結果の記録先
```

## Windowsで起動する

この試作の比較対象はWindows x64です。Rustとネイティブビルド環境が必要です。MSVC環境ではVisual Studio 2022のC++ x64/x86ビルドツール、Windows SDKを用意し、Developer PowerShellから実行してください。Servoには追加のネイティブ依存があります。[Servo 0.7.0のWindows手順](https://github.com/servo/servo/blob/v0.7.0/README.md#windows)（ATL、`mach bootstrap`）に従って準備します。[Servo Book](https://book.servo.org/building/building.html)も参照してください。

Rust 1.99.1で依存関係を解決しました。エンジンの直接依存は完全一致の版指定、推移依存は各`Cargo.lock`で固定しています。Servo 0.7.0公式の基準ツールチェーンは1.97.1ですが、このロックファイル全体を1.97.1では検証していません。全構成の検証状況は[検証記録](docs/validation.md)を確認してください。

リポジトリのルートで、1つずつ実行します。

```powershell
cargo run --release --locked --manifest-path probes/servo/Cargo.toml
cargo run --release --locked --manifest-path probes/blitz/Cargo.toml
cargo run --release --locked --manifest-path probes/webview2/Cargo.toml
```

WebView2版には[WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)が必要です。GNU向けビルドでは、取得済み`webview2-com-sys`クレートの`x64/WebView2Loader.dll`も実行ファイルの隣に配置します（MSVC版は静的リンク）。ServoとWebView2はソースツリー内の`ui/index.html`を`file://`で読みます。実行ファイルだけの配布にはまだ対応していません。BlitzはCSSをビルド時に取り込むので、CSS変更後は再ビルドします。

[操作と性能の比較手順](docs/comparison.md)に従い、[実測表](measurements/README.md)へ記録してください。現時点では日本語IMEの実機結果と性能比較は未確定です。

IPC、保存、画面遷移、エンジン切り替え、フォールバック、プラグイン、クラッシュ復旧は実装していません。入力・描画と実測の結果から構成を選んだ後に、必要な機能だけ追加します。

## 実装で参照した公式コード

- [Servo 0.7.0の最小winitサンプル](https://github.com/servo/servo/blob/v0.7.0/components/servo/examples/winit_minimal.rs)。派生したServoホストのファイルはMPL-2.0です。
- [Dioxus Native 0.7.10](https://github.com/DioxusLabs/dioxus/tree/v0.7.10/packages/native)。ネイティブ描画の`launch_cfg`を使用します。
- [wry 0.57.1](https://docs.rs/wry/0.57.1/wry/)。WindowsではWebView2を使用します。
