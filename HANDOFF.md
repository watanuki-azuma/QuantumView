# QuantumView handoff — 2026-11-09 JST

## 今回の終了状態

ユーザーの「それが終わったらhandoff書いて終わりにして」に従い、ここで終了する。最新のスクロール修正版への実機返答は「移動量が少なすぎる、まあいいです一旦」。**スクロール改善は未完了**で、追加調整・追加ビルド・性能比較・メモ保存機能へ進まない。

## 目的と進め方

- 最終目的は、ChromiumやOS標準WebViewを使わずにHTML/CSS/**JavaScript**を組み込める、小さなWebView代替ホスト。
- 単一メモの編集・自動保存・復元・配布を成立させ、その実装で使ったホスト処理だけを切り出す。[ロードマップ](docs/roadmap.md)参照。
- Servo 0.7.0が実装候補。WebView2は比較基準でありフォールバックではない。
- Blitz/Dioxus Nativeは既存HTML/JSのWebView置き換えに合わないため、ユーザーの指示で試作・起動手順・候補から削除済み。過去の測定記録だけ保持。
- KISS/YAGNI。バックエンド抽象化、プラグイン、互換層、クラッシュ復旧、将来用API、テスト基盤は増やさない。mainの更新・pushはユーザーが許可済み。

## 実装と確認結果

現在は`probes/servo`と`probes/webview2`の独立したCargoプロジェクト。同じ`ui/index.html`・`style.css`・`app.js`を表示する。比較用UIは変更していない。JS–Rust通信、保存、復元、配布用のリソース読込はまだない。

| 変更 | 状態 |
| --- | --- |
| Windows OpenGL初期化 | `vendor/servo-paint-api`に公式修正を適用。起動パニック解消 |
| IME確定で編集欄のフォーカスが外れる | `Ime::Disabled`から`ImeEvent::Dismissed`へ転送する経路を除去。実機再確認済み |
| 未確定文字のBackspaceで表示が残る | `vendor/servo-script`の`text_input.rs`だけ修正。実機で1回のBackspaceによる削除・変換・取消・連続入力を確認済み |
| 文字数・クリア、項目100との往復、狭い幅の1列表示 | IME修正版でユーザー確認済み |
| ホイールの段階的な移動 | 下記を実装・ビルド。実機では移動量不足が残り、未完了 |

最新のホイール修正は`vendor/servo-paint`へ限定したCargoパッチ。ホストは行入力を`DeltaLine`、ピクセル入力を`DeltaPixel`として渡す。元のwheelイベントがJavaScriptで処理され、`preventDefault()`されなかった行入力だけを、既存のrefresh driverで120msに分ける。計算上は従来と同じ40論理ピクセル/行。ピクセル入力は即時反映する。

補間中の同方向入力は合算し、反対方向の残りは破棄する。キー・マウスボタン・タッチ・明示的なスクロールで補間を止め、完了後はobserverを解除する。公開API・依存版は変更していない。公開版14ファイルのうち変更は`lib.rs`と`webview_renderer.rs`だけ。非公開の`wheel_scroll.rs`を追加した。[パッチ説明](probes/servo/vendor/servo-paint/README.quantumview.md)参照。

**次に調べるべき点は、実際の描画処理を通ったホイール1回分の累積移動量。** 補間計算の距離保存テストだけでは実機の移動量を保証できない。係数を増やす前に、残りの移動量が最後まで反映されるかを確認する。最新バイナリーでの日本語入力の回帰も未確認。今回の返答から合格を推測しない。

## ビルド・バイナリー

- 最新releaseは`--release --frozen -j2`で**1回成功、1分4秒**。テスト4件も成功（距離保存、追加入力、軸ごとの方向反転、フレーム遅延）。実際の`wheel_scroll.rs`を`rustc --test`で直接テストし、別Cargoプロジェクトは作っていない。
- ロック内822パッケージの名前・版・依存先は不変。今回のlock差分は`servo-paint`のregistry source/checksumの削除のみ。
- 上流由来のservo-script 53件・servo-paint 1件の警告あり。無関係な上流コードの整形や`cargo fix`はしない。
- 最新バイナリーSHA-256: `B48F6FDD9E757FE75B59743FC39DFBDB46C16ECE0EC97F678F7ACC644C87EC6A`。
- 直前のIME実機合格版SHA-256: `E6DF34527416FC868E25BCF561AA220A40C916CEFACBDA119727DBE93CFFF26A`。

このPCの作業ルートは`C:\Users\JIU\Documents\Codex\2026-11-08\github-plugin-github-openai-curated-remote-2`。リポジトリはその`work\QuantumView`。

- 最新exe: `work\servo-scroll-fixed\quantumview-servo.exe`（起動時PID 24960。停止する場合はPIDを信用せず実行パスを再確認）。
- IME合格版exe: `work\servo-preedit-fixed\quantumview-servo.exe`。
- 比較用の元バイナリー: `work\frozen-probes\`。上書きしない。
- ビルド手順: `work\rebuild-servo-scroll.ps1`。ログ: `work\servo-scroll-build.log`。
- GUI起動は`Start-Process -NoNewWindow`とログへのリダイレクトを使う。余分なconhostのCPUが測定へ混ざった経緯がある。

ビルドはRust/Cargo 1.99.1の`stable-x86_64-pc-windows-msvc`、VS Build Tools 2022（MSVC 14.44、SDK 10.0.26100.0）、LLVM 23.1.5、Python 3.13.17。GNUのデフォルト設定は変えていない。同じ`probes/servo/target`を使い、清掃・依存更新・別プロジェクトでの検証ビルドをしない。今回成功した環境ではPythonの実体をPATHへ置き、`PYTHON`と`PYTHON3`は未設定にしている。これを変更するとキャッシュが無効になる可能性がある。

## 残課題と計測

- スクロールの移動量不足と最新バイナリーの入力回帰確認が先。右クリックメニューは未実装。
- IME候補位置、全編集キー、複数行・絵文字の全ショートカット、トラックパッド、最小化復帰・DPIは未確認。
- 起動時はビルド時の絶対ソースパスからUIを読む。exeだけで配布可能とは言わない。
- [既存のアイドル測定](measurements/README.md)はコミット`80b45e15cbbf583cc57390f8855e1d297744481f`の固定バイナリーの結果。後続修正版の値に置き換えない。軽量性・採用の最終結論は出ていない。
- 今回のスクロール負荷比較は中断。`work\scroll-session`のServo 2回分のCSVは作業用記録で、WebView2の比較値はない。モデルの返答を終了合図にして手動操作を長く続けさせたため、比較結果に採用しない。
- ネイティブ画面取得は`FrameArrived timed out`が続いた。ツール修復や別検証アプリのビルドへ脱線せず、必要な実機確認を短く行う。30秒操作をモデルの返答で区切る方式は繰り返さない。

詳細は[検証記録](docs/validation.md)。次の作業はユーザーから再開を依頼されてから行う。

---

## 追記 — 2026-11-14（Phase 1 開始）

ユーザーの再開依頼を受け、改訂版ロードマップ（Phase 1 = Servo継続可否の先行判定ゲート）で作業を再開した。ホイール移動量不足の切り分けは静的に完了し、原因は winit 0.30.13 が Windows の「1ノッチの行数」設定を掛けない単位解釈である可能性が最も高い（補間・適用経路の距離保存はコードで確認）。計装ブランチ `fix/wheel-scroll-trace` と修正案ブランチ `fix/wheel-lines-per-notch` を用意済みで、いずれも未マージ。最終確定と Go/No-Go 判定は実機セッション（トレース取得 → 修正A/B → 計測 → 入力チェック）待ち。詳細は検証記録の「Phase 1（Servo継続可否の先行判定）の途中記録 — 2026-11-14」。
