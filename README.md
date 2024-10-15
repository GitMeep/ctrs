# CT-RS

A "real-time" backprojection volumetric CT viewer written in Rust.

## Using

Click the "open" button and select a scan descriptor `.json` file. Example datasets can be found in the [data](./data/) directory. Use threshold to only display samples above that value.

## Building

Build using `cargo`:

```bash
cargo build --release
```

The executable can then be found in `target/release/`.

To build and run immediately, use:

```bash
cargo run --release
```

## Environment variables

WGPU environment variables can be used to control the graphics backend used (Vulkan, OpenGL, etc.), which graphics adapter is used, etc. See the [WGPU readme](https://github.com/gfx-rs/wgpu?tab=readme-ov-file#environment-variables).

To control logging, use the `RUST_LOG` environment variable. See [env_logger](https://docs.rs/env_logger/latest/env_logger/) (you must run CTRS from the terminal to see logs).

As an example, run with Vulkan, high-power adapter and info logging:

```bash
WGPU_POWER_PREF=high WGPU_BACKEND=vulkan RUST_LOG=info ct-rs
```

_Note:_
Iced does not seem to like low framerates when running with OpenGL, so Vulkan is recommended.
The version of WGPU that iced 0.13.1 uses seems to crash when running Vulkan with Wayland (at least on NVIDIA) so when running NVIDIA on Linux it is recommended to use X11 for now. (Should be fixed in the next iced release, see [this issue](https://github.com/iced-rs/iced/issues/2572)).
