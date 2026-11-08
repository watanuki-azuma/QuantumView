# 検証記録

実施日: 2026-11-08（日本時間）。これは初期試作の検証記録です。全構成の動作保証や性能比較の完了を示すものではありません。

## 追加したWindowsビルド環境

初回のGNU環境での停止後、以下をインストールし、それぞれ実行・検出できることを確認しました。再起動は不要でした。

- Visual Studio Build Tools 2022 17.14.44: MSVC v143（14.44）、ATL、Windows SDK 10.0.26100.0、CMake 3.31.6とNinja。
- LLVM 23.1.5（`clang-cl`、`libclang.dll`）。
- Python 3.13.17。
- Rust / Cargo 1.99.1の`stable-x86_64-pc-windows-msvc`ツールチェーン。

既存のGNUツールチェーンとデフォルト設定は保持しています。MSVCのDeveloper PowerShell環境で、LLVMとPythonの実体をPATHへ追加し、`LIBCLANG_PATH`・`PYTHON`・`PYTHON3`・`CC=clang-cl`・`CXX=clang-cl`をビルドプロセス内で設定します。実行コマンドは[README](../README.md#windowsで起動する)に記載しています。

## MSVCでのビルドとOpenGL初期化の修正

元の依存構成で`cargo +stable-x86_64-pc-windows-msvc build --release --locked -j 2 --manifest-path probes/servo/Cargo.toml`が成功しました。初回ビルド時間は29分51秒で、アプリ本体のコンパイルとリンクまで完了しています。

起動時には`glow`が`GL_VERSION`を読み取る際にパニックになりました。これはOpenGLコンテキストをcurrentにする前にGL関数を読み込むServo 0.7.0側の初期化順序の問題で、[上流コミット16e4b15（#48636）](https://github.com/servo/servo/commit/16e4b1591e1fa4a47016643e34d4f259185c5f2f)に同じ問題の修正があります。

その修正のみを`probes/servo/vendor/servo-paint-api`へ取り込み、Cargoの`[patch.crates-io]`で適用しました。公開API・描画方式・Servoの固定版は変更していません。公開クレートとの比較で、Rustソースの変更は上流と同じ初期化順序の修正だけであることを確認しています。

修正適用後の`--release --locked -j 2`ビルドも成功しました（再ビルド10分29秒）。実行時のOpenGLパニックは解消し、`QuantumView - Servo`ウィンドウが生成され、プロセスが継続して動くことを確認しました。Alt+F4で終了し、終了コード0になることも確認しました。

Windows操作ヘルパーの画面取得は`FrameArrived timed out`で失敗したため、描画の外観と日本語IMEの動作は未確認です。起動ログにはGPU選択シンボル、最適化シェーダー、ClientStorage用一時DB作成についての警告があり、終了時にはメモリープロファイラーの切断警告も出ています。これらの影響と永続化は今回の起動確認では評価していません。

## 固定版Servoのユーザーによる実機確認

ロードマップの段階1として、固定済みServoを再ビルドせずに起動し、ユーザーの操作で次を確認しました。

| 項目 | 結果 |
| --- | --- |
| 日本語メモとカードの表示 | 表示されるとの報告。スクリーンショットと100件すべての目視確認は未取得 |
| 日本語変換・確定・Esc取消 | 操作できるとの報告 |
| 確定後の連続入力 | 不合格。続けて入力するとカーソルが消え、確定時に文字は入る |
| Ctrl+C / Ctrl+V | コピーと貼り付けができるとの報告。複数行・全ショートカット・候補位置は未確認 |
| 右クリックメニュー | 表示されない。現在のホストでは未対応 |

Windows版winit 0.30.13では、通常の`WM_IME_ENDCOMPOSITION`でも`Ime::Disabled`を送ります。一方、Servo 0.7.0の`ImeEvent::Dismissed`は入力欄から文書のビューポートへフォーカスを移します。従来のホストは両者を結び付けていたため、正常な変換終了でも編集欄をぼかす経路がありました。

この経路を除去し、`Ime::Disabled`では変換中フラグだけを解除する最小修正を実装しました。`--release --frozen -j2`のビルドは9分30秒で成功し、rustfmtの確認も成功しています。エンジンや推移依存は変更せず、比較用固定バイナリーも保持しています。

修正版のSHA-256は`3B8C92CBD194D0FBC3FBB4C25FE39F99093D0F5831DCF6451BC1EE8D0FD57D0C`です。ユーザーの再確認で、連続変換・取消・フォーカス移動は基本的に問題ないとの報告を得ました。ただし、未確定の「h」をBackspaceで消すと1回では表示が残り、もう1回押すと消える不具合が残っています。

## 初回検証の環境

- Windows x64、Rust / Cargo 1.99.1、`x86_64-pc-windows-gnu`。
- 元の環境にはC/C++コンパイラーとMSVCビルドツールがありません。
- 検証用にGNU binutilsを作業フォルダーへ展開。システム設定とリポジトリのビルド設定は変更していません。
- ビルドと画面操作の確認中は他のビルドも動いていたため、性能測定の条件として使いません。

## 初回GNU環境でのビルドと静的確認

| 確認 | Servo | Blitz | WebView2 |
| --- | --- | --- | --- |
| 固定版とロックファイル | 0.7.0 | Dioxus Native 0.7.10 | wry 0.57.1 |
| `cargo fmt -- --check` | 成功 | 成功 | 成功 |
| `cargo metadata --no-deps --locked --offline` | 成功 | 成功 | 成功 |
| `cargo check --locked` | ネイティブ依存で停止 | 成功 | 成功 |
| 実行ファイルのビルド | 未完了 | dev成功（`--locked -j 2`） | release成功 |

Servoの`cargo check`は、`glslopt 0.1.14`のビルド時に`cc-rs: failed to find tool "gcc.exe"`で停止しました。アプリ本体のコンパイルには到達していません。`servo 0.7.0`の公開クレートのVCS情報は公式タグのコミット`aac43a3f31a259f04a574f5ec4e959c943ad7cc7`と一致しており、入力・描画・IMEの呼び出しはその版のソースと照合しています。これはコンパイラーによる検証の代用にはなりません。

Dioxus Native 0.7.10には、`Config::default`で使う`dioxus_cli_config`の依存が`hot-reload`機能の内側にあるという問題があり、最初の`default-features = false`では上流クレートがコンパイルできませんでした。この依存を有効にするために`hot-reload`だけ追加しています。独自の互換レイヤーや上流のフォークは導入していません。開発サーバーへの接続処理は上流で`cfg(debug_assertions)`に限定され、releaseビルドでは実行されません。[該当版の公式ソース](https://github.com/DioxusLabs/dioxus/blob/v0.7.10/packages/native/src/lib.rs)を参照してください。

ロックファイル内のBlitzは`blitz-dom 0.2.4`、`blitz-shell 0.2.3`、`blitz-paint 0.2.1`、描画は`vello 0.6.0`です。WebViewやChromiumへのフォールバックはありません。

## 比較対象から除外したBlitzの検証履歴

初回のGNU環境ではdev実行ファイルのビルドに成功し、`QuantumView - Blitz`ウィンドウが生成されることを確認しました。その後、MSVCで`--release --locked -j2`のビルドにも成功しました（10分11秒）。Windows操作ヘルパーで画面キャプチャーがタイムアウトし、現在の最小構成のアクセシビリティ情報にはウィンドウ枠しか現れないため、画面内容と入力操作の実機検証は未完了です。

その後のユーザー実機確認では、日本語メモとカードの表示・変換・確定・取消・連続入力・Ctrl+C/Vは動くとの報告を得ました。一方、IME候補がウィンドウ内の入力位置ではなくディスプレイ右下へ出ることと、入力中の文字が豆腐になる不具合が報告されています。

Blitz/Dioxus NativeはUIロジックをRustで書く構成で、既存のHTML/JavaScriptを実行するWebViewの置き換えには合いません。ユーザーの指示に従って試作コードと検証候補から除外し、検証プロセスも終了しました。この節と既存の測定値は過去の記録です。

## WebView2の起動確認

MSVCでも`--release --locked -j2`のビルドに成功しました（1分2秒）。MSVC版はLoaderを静的リンクします。

release実行ファイルを起動し、Windowsのアクセシビリティ情報から次を確認しました。

- ウィンドウ`QuantumView - WebView2`が生成される。
- `ui/index.html`が`file://`で読み込まれる。
- 日本語のタイトル、メモの初期値、36文字のカウント、項目1〜100が存在する。
- UI Automationで入力値を「日本😀」へ変更すると、`input`処理が動き3文字になる。
- Alt+F4後に検証用ウィンドウが消える。

GNU版では`WebView2Loader.dll`が必要でした。取得済み`webview2-com-sys 0.39.1`の`x64/WebView2Loader.dll`をビルド出力へ配置して起動しました。DLLはリポジトリに含めていません。

画面キャプチャーはWindows操作ヘルパーの`window capture timed out`で取得できませんでした。したがって、CSSの外観、文字の欠け、リサイズ後の描画品質は確認済みとはしていません。UI Automationでの入力は日本語IMEによる変換試験でもありません。

## 共通画面と計測スクリプト

- `node --check ui/app.js`: 成功。
- 一時的なNodeのDOMスタブで、入力によるカウント更新、「日本😀」=3、クリア=空欄/0、カード100件と先頭/末尾の番号を確認。
- PowerShellパーサーによる`measure.ps1`の構文確認: 成功。
- PowerShell自身を対象に2秒間の計測を実行し、プロセス数・メモリ・CPUを含む2行のCSV出力を確認。これは計測器の動作確認で、候補エンジンの性能値ではありません。
- GPU列の追加後も、起動中のServoを対象とする2秒間の動作確認で専用/共有メモリの数値を取得。その後の有効9回の測定では、全行のGPU値・CPU値が存在し、プロセス増減がないことを確認しました。

## 残っている実機確認

固定したMSVC release実行ファイルで、各構成3回のアイドル時メモリ・CPU・GPUメモリを測定しました。結果と条件は[実測表](../measurements/README.md)に記録します。

全構成の描画の外観・日本語IME変換・候補位置・メモ帳とのクリップボード交換・スクロール・リサイズ・DPI、コールド起動とスクロール負荷は未確認です。Windows操作ヘルパーは`FrameArrived timed out`で画像を取得できず、標準の画面コピーをPaintへ保存する方法も完了できませんでした。

画面取得の代替として別Cargoプロジェクトからエンジン公式APIを呼ぶ作業用検証を試しましたが、Servoの依存部分が再コンパイルされました。ユーザーの指示に従ってそのビルドを中止し、検証用コードはリポジトリに含めていません。以後の測定ではビルド済み3本のSHA-256を固定し、コンパイラーが動いていない状態で実施しています。エンジンの描画量と入力が同等であることは未確認のため、採用の結論は出していません。
