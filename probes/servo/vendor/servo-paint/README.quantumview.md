# Local patch to servo-paint 0.7.0

Source: the published `servo-paint 0.7.0` crate, Servo commit
`aac43a3f31a259f04a574f5ec4e959c943ad7cc7`, `components/paint`.
MPL-2.0 headers and the normalized manifest are retained. Registry metadata,
the original manifest and the nested lockfile are omitted.

The renderer spreads the default action of accepted line-mode wheel input
over 120 ms using its existing refresh driver. It preserves 40 logical pixels
per line, dispatches the original wheel event once, respects preventDefault(),
and leaves pixel-mode input immediate. Reversing an axis discards its old
remaining movement. Keyboard, mouse-button, touch and explicit scrolling
cancel the pending animation. The refresh observer is removed when finished.

Only `lib.rs` and `webview_renderer.rs` differ from the published Rust files.
`wheel_scroll.rs` contains the private interpolation and four focused tests;
it has no external dependencies and can be tested directly with rustc.
The full crate is present because Cargo patches apply at package level.
No engine version, dependency version or public API is added or changed.
