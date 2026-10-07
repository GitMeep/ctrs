build:
    cargo build

debug:
    WGPU_BACKEND=vulkan WGPU_POWER_PREF=high RUST_LOG=info cargo run

# renderdoc: build
#     WGPU_BACKEND=vulkan WGPU_POWER_PREF=high RUST_LOG=info WAYLAND_DISPLAY="" renderdoccmd capture -w ./target/debug/ct-rs

qrenderdoc: build
    qrenderdoc ./ctrs.cap