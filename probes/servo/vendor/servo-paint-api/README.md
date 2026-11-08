# Servo paint API 0.7.0 — OpenGL initialization fix

Source: the published `servo-paint-api 0.7.0` crate, licensed MPL-2.0.
The original source headers and manifest are retained. Cargo's registry metadata,
the standalone lockfile, and the workspace-only original manifest are omitted.

The only Rust source change backports Servo commit
[`16e4b1591e1fa4a47016643e34d4f259185c5f2f`](https://github.com/servo/servo/commit/16e4b1591e1fa4a47016643e34d4f259185c5f2f)
(`paint: Make the GL context current before creating the glow context`, #48636).

On Windows WGL, the published version calls glow before making the OpenGL
context current, which panics while reading `GL_VERSION`. This fix makes the
new context current before loading GL functions. It retains the pinned Servo
version and rendering backend; there is no fallback or new public API.
