# vpinball-plugin-rust

Example of a Visual Pinball plugin written in Rust

## Visual Pinball Plugin system

This is still work in progress and the API is not stable yet. All documentation is currently
at https://github.com/vpinball/vpinball/blob/10.8.1/src/plugins/VPXPlugin.h

## Installing the plugin

```sh
# build the plugin (this will also download the vpinball plugin header)
cargo build
# set the vpinball folder location env var
export VPINBALL_FOLDER=$HOME/vpinball
# copy the plugin to the plugin folder
mkdir -p $VPINBALL_FOLDER/plugins/vpinball_plugin_fps
# Mac
cp target/debug/libvpinball_plugin_fps.dylib $VPINBALL_FOLDER/plugins/vpinball_plugin_fps
# Linux
cp target/debug/libvpinball_plugin_fps.so $VPINBALL_FOLDER/plugins/vpinball_plugin_fps

cp fpscounter/plugin.cfg $VPINBALL_FOLDER/plugins/vpinball_plugin_fps
```

### Setting up the plugin

Add the following section to the `$HOME/.vpinball/VPinballX.ini` config file

```ini
[Plugin.fps]
enable = 1

[Plugin.rainbow.dmd]
enable = 1
```

## Issues tracked on the vpinball repo

* https://github.com/vpinball/vpinball/issues/2008
