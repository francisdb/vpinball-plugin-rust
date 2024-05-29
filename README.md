# vpinball-plugin-rust

Example of a Visual Pinball plugin written in Rust

## Visual Pinball Plugin system

This is still work in progress and the API is not stable yet. All documentation is currently
at https://github.com/vpinball/vpinball/blob/10.8.1/src/plugins/VPXPlugin.h

## Installing the plugin

```sh
# build the plugin (this will also download the vpinball plugin header)
cargo build
# copy the plugin to the plugin folder
mkdir -p ~/.vpinball/plugins/vpinball_plugin_rust
# Mac
cp target/debug/libvpinball_plugin_rust.dylib ~/.vpinball/plugins/vpinball_plugin_rust
# Linux
cp target/debug/libvpinball_plugin_rust.so ~/.vpinball/plugins/vpinball_plugin_rust

cp plugin.cfg ~/.vpinball/plugins/vpinball_plugin_rust
```

## Header not being 100% C compatible

Without `-x c++` bindgen will complain about these things:

* Nested `typedef` are not supported in C, so you need to make `ViewSetupDef` and `TableInfo` top-level.
* `const OptionUnit unit` needs to be changed to `const enum OptionUnit unit`.
* Add a `#include <stdbool.h>` to the header file. (only for C89)
