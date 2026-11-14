# ホイール移動量のトレース

スクロール修正版で報告された「ホイール1ノッチの移動量が少なすぎる」について、係数を変える前に、実際の描画経路を通った累積移動量を数えるための計装。環境変数 `QUANTUMVIEW_WHEEL_TRACE=1` のときだけ、標準エラーへ `[wheel-trace]` 付きの行を出す。通常時は無効で、動作は変わらない。

## 記録されるもの

| 行 | 意味 |
| --- | --- |
| `host wheel: mapped=(x, y, line)` | ホスト（winit）が受けたホイール入力そのもの。1ノッチで何行として届いているか |
| `accepted line wheel: lines=…, line_size=…, device_delta=…` | JSで処理されず受理された行入力と、40px/行×HiDPI換算後の要求量（デバイスピクセル） |
| `start_wheel_scroll: device_delta=…` | 補間へ渡された量 |
| `add: …` / `advance: fraction=…, step=…, remaining=…, finished=…` | 補間の加算と各フレームの適用量。stepの合計が要求量と一致するか |
| `applied: new_offset=(x, y)` / `applied: none` | 実際にスクロールノードへ適用された結果のオフセット（レイアウトピクセル）。連続する行の差が実移動量 |

## 取り方

Developer PowerShellで、既存のビルド手順どおりreleaseをビルドしてから実行する。

```powershell
$env:QUANTUMVIEW_WHEEL_TRACE = '1'
.\quantumview-servo.exe 2> wheel-trace.log
```

手順は短く保つ。

1. ページ本文の上、スクロール範囲の上下端から離れた位置にカーソルを置き、止めたままにする。
2. ホイールを下方向へ**1ノッチだけ**回し、1秒待つ。
3. 続けて3ノッチを普通の速さで回し、1秒待つ。
4. 終了して `wheel-trace.log` を共有する。

## 読み方（1ノッチ分）

- **単位の解釈が原因**: `host wheel` の合計が `y=-1`（1行）で、`accepted` の要求が `40 × HiDPI`、`advance` のstep合計＝要求量、`applied` の増分＝要求量÷表示倍率。経路は距離を保っており、最初の要求量自体が小さい。この場合は「1ノッチ＝1行」という前提側を直す。
- **適用側の損失**: 要求とstep合計は一致するのに、`applied` の増分が換算後の要求より小さい（または `none` が混ざる）。クランプ、対象ノード違い、軸の感度などが候補になる。
- **進行の停止**: `advance` が `remaining` を残したまま止まる、または `applied` が出てこない。タイマー起床と描画のどちらで止まったかを次に見る。

日本語入力の回帰確認は、このトレースとは別に最新バイナリーで短く行う。
