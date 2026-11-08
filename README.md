# QuantumView

ChromiumやOS標準WebViewに依存せず、HTML/CSS/JavaScriptのローカルUIをアプリへ組み込める、小さなWebView代替ホストを目指します。Servoを実装候補、WebView2を比較基準として検証しています。

メモアプリを実用サンプルとして成立させ、その実装で必要な描画・入力・ホスト連携だけを切り出します。メモと別の最小画面で同じホストを利用できるところまでを[ロードマップ](docs/roadmap.md)に記載しています。

同じ大きさのウィンドウに、日本語メモ、文字数、クリアボタン、100件のカードを表示します。入力、IME、クリップボード、スクロール、リサイズと実際のリソース使用量を比較します。

| 実行ファイル | 描画 | UIロジック | 用途 |
| --- | --- | --- | --- |
| `quantumview-servo` | Servo 0.7.0 / WebRender | ローカルHTML/CSS/JS | 候補 |
| `quantumview-webview2` | Windows WebView2（Chromium） | Servoと同じHTML/CSS/JS | 比較基準 |

2つは独立した小さなCargoプロジェクトで、同じHTML/CSS/JavaScriptを表示します。エンジン固有の入力処理を比較するため、それぞれ直接起動します。WebView2だけをビルドする場合はServoの取得は不要です。

```text
probes/servo/       Servoのウィンドウ・描画・入力
probes/webview2/    wryによるWindows比較版
ui/                共通CSS、Servo/WebView2用HTMLとJS
scripts/measure.ps1 プロセス群のメモリ、CPU、GPUメモリをCSVに記録
docs/              比較手順、検証結果
measurements/      実測結果の記録先
```

## Windowsで起動する

この試作の比較対象はWindows x64です。RustのMSVC版、Visual Studio 2022のC++ x64/x86ビルドツール・ATL・Windows SDK、LLVM、Python 3.11以降を用意し、新しく開いたDeveloper PowerShellから実行してください。WindowsAppsのPythonエイリアスだけではコード生成を実行できません。[Servo 0.7.0のWindows手順](https://github.com/servo/servo/blob/v0.7.0/README.md#windows)と[Servo Book](https://book.servo.org/building/building.html)も参照してください。

GNU版Rustを使っている場合は、既存の設定を残してMSVC版を追加できます。

```powershell
rustup toolchain install stable-x86_64-pc-windows-msvc --profile minimal
```

Rust 1.99.1で依存関係を解決しました。エンジンの直接依存は完全一致の版指定、推移依存は各`Cargo.lock`で固定しています。Servo 0.7.0公式の基準ツールチェーンは1.97.1ですが、このロックファイル全体を1.97.1では検証していません。全構成の検証状況は[検証記録](docs/validation.md)を確認してください。

公開クレートへの修正は`probes/servo/vendor/`へ置き、Cargoのパッチで適用しています。エンジンの版と描画方式は維持しています。

- `servo-paint-api 0.7.0`: WindowsのOpenGL初期化パニックに対する[Servo公式の初期化順序の修正](https://github.com/servo/servo/commit/16e4b1591e1fa4a47016643e34d4f259185c5f2f)。
- `servo-script 0.7.0`: 未確定文字をBackspaceで空にした際、その更新が無視されて表示が残る問題への[1ファイルの修正](probes/servo/vendor/servo-script/README.quantumview.md)。

LLVMの標準インストール先が`C:\Program Files\LLVM`の場合、リポジトリのルートで以下を実行します。Pythonの実体がPATHに含まれていることも確認してください。各アプリは1つずつ起動します。

```powershell
$env:PATH = 'C:\Program Files\LLVM\bin;' + $env:PATH
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
$env:CC = 'clang-cl'
$env:CXX = 'clang-cl'
cargo +stable-x86_64-pc-windows-msvc run --release --locked --manifest-path probes/servo/Cargo.toml
cargo +stable-x86_64-pc-windows-msvc run --release --locked --manifest-path probes/webview2/Cargo.toml
```

WebView2版には[WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)が必要です。GNU向けビルドでは、取得済み`webview2-com-sys`クレートの`x64/WebView2Loader.dll`も実行ファイルの隣に配置します（MSVC版は静的リンク）。ServoとWebView2はソースツリー内の`ui/index.html`を`file://`で読みます。実行ファイルだけの配布にはまだ対応していません。

[操作と性能の比較手順](docs/comparison.md)に従い、[実測表](measurements/README.md)へ記録してください。固定したMSVC release実行ファイルで、各構成3回のアイドル時メモリ・CPU・GPUメモリを測定済みです。後続のServo修正版では、日本語変換・取消・連続入力・未確定文字のBackspaceとCtrl+C/Vの実機確認を得ています。描画の同等性、スクロールの操作感と負荷などは確認が残るため、採用の判断は保留しています。

IPC、保存、画面遷移、エンジン切り替え、フォールバック、プラグイン、クラッシュ復旧は実装していません。入力・描画と実測の結果から構成を選んだ後に、必要な機能だけ追加します。

## 実装で参照した公式コード

- [Servo 0.7.0の最小winitサンプル](https://github.com/servo/servo/blob/v0.7.0/components/servo/examples/winit_minimal.rs)。派生したServoホストのファイルはMPL-2.0です。
- [wry 0.57.1](https://docs.rs/wry/0.57.1/wry/)。WindowsではWebView2を使用します。
