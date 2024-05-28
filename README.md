# vpinball-plugin-rust

Example of a Visual Pinball plugin written in Rust

## Visual Pinball Plugin system

This is still work in progress and the API is not stable yet. All documentation is currently at

## Installing the plugin

```sh
# get the latest header file from the vpinball repo
./download_header.sh
# build the plugin
cargo build
# copy the plugin to the plugin folder
mkdir -p ~/.vpinball/plugins/vpinball_plugin_rust
# Mac
cp target/debug/libvpinball_plugin_rust.dylib ~/.vpinball/plugins/vpinball_plugin_rust
# Linux
cp target/debug/libvpinball_plugin_rust.so ~/.vpinball/plugins/vpinball_plugin_rust

cp plugin.cfg ~/.vpinball/plugins/vpinball_plugin_rust
```