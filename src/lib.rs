mod fpscounter;
mod plugin;

use crate::plugin::{DoPluginLoad, DoPluginUnload, VPXPlugin, VPXPluginAPI};

#[no_mangle]
pub extern "C" fn PluginLoad(api: *mut VPXPluginAPI) {
    DoPluginLoad(api, on_load, on_unload);
}

#[no_mangle]
pub extern "C" fn PluginUnload() {
    DoPluginUnload();
}

fn on_load(plugin: &mut VPXPlugin) {
    println!("Plugin loading");
    plugin.subscribe_event("VPX.OnGameStart", Box::new(on_game_start));
    plugin.subscribe_event("VPX.OnGameEnd", Box::new(on_game_end));
    plugin.subscribe_event("VPX.OnPrepareFrame", Box::new(on_prepare_frame));
}

fn on_unload(plugin: &mut VPXPlugin) {
    println!("Plugin unloading");
    plugin.unsubscribe_event("VPX.OnGameStart");
    plugin.unsubscribe_event("VPX.OnGameEnd");
    plugin.unsubscribe_event("VPX.OnPrepareFrame");
}

fn on_game_start(plugin: &VPXPlugin, event_id: u32) {
    println!("plugin event {event_id}: Game is starting");
    // Game is starting (plugin can be loaded and kept alive through multiple game plays)
    // After this event, all functions of the API marked as 'in game only' can be called

    let setup = plugin.get_active_view_setup();
    println!("Active view setup: {setup:?}");

    let table = plugin.get_table_info();
    println!("Active table: {}", table.path);

    plugin.push_notification("Hello World", 5000);
}

fn on_game_end(_plugin: &VPXPlugin, event_id: u32) {
    println!("plugin event {event_id}: Game is ending");
    // Game is ending
    // After this event, all functions of the API marked as 'in game only' may not be called anymore
}

fn on_prepare_frame(_plugin: &VPXPlugin, _event_id: u32) {
    // TODO add fps counter
    // Called when the player is about to prepare a new frame
    // This can be used to tweak any visual parameter before building the frame (for example head tracking,...)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_load_unload() {
        let mut api = plugin::tests::TestVPXPluginAPI::init();
        PluginLoad(&mut api);

        PluginUnload();
    }
}
