/// Our example plugin for Virtual Pinball
mod fpscounter;
mod plugin;

use crate::plugin::{VPXPlugin, VPXPluginAPI};
use log::info;
use std::rc::Rc;
use std::sync::Mutex;

/// Everything should be called from a single thread that originates on the vpinball side.
static mut PLUGIN: Option<Rc<VPXPlugin>> = None;

pub fn get_plugin() -> Rc<VPXPlugin> {
    unsafe { PLUGIN.as_ref().expect("Plugin not loaded").clone() }
}

#[no_mangle]
pub extern "C" fn PluginLoad(api: *mut VPXPluginAPI) {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let mut plugin = VPXPlugin::new(on_load, on_unload);
    plugin.load(api);
    unsafe {
        PLUGIN = Some(Rc::new(plugin));
    }
}

#[no_mangle]
pub extern "C" fn PluginUnload() {
    unsafe {
        if let Some(plugin) = PLUGIN.take() {
            match Rc::try_unwrap(plugin) {
                Ok(mut plugin) => {
                    plugin.unload();
                }
                Err(_) => {
                    panic!("Failed to get mutable reference to plugin");
                }
            }
        }
    }
}

fn on_load(plugin: &mut VPXPlugin) {
    info!("Plugin loading");
    let fps_counter = Rc::new(Mutex::new(fpscounter::FPSCounter::new()));
    let fps_counter_clone = Rc::clone(&fps_counter);
    plugin.subscribe_event("VPX.OnGameStart", Box::new(on_game_start));
    plugin.subscribe_event(
        "VPX.OnGameEnd",
        Box::new(move |event_id| {
            info!("plugin event {event_id}: Game is ending");
        }),
    );
    plugin.subscribe_event(
        "VPX.OnPrepareFrame",
        Box::new(move |_event_id| {
            let mut fps_counter = fps_counter_clone.lock().unwrap();
            let fps = fps_counter.update();
            if let Some(fps) = fps {
                info!("FPS: {:.2}", fps);
            }
        }),
    );
    plugin.subscribe_event(
        "VPX.OnSettingsChanged",
        Box::new(move |_event_id| {
            info!("Settings changed");
        }),
    );
}

fn on_unload(_plugin: &mut VPXPlugin) {
    info!("Plugin unloading");
}

fn on_game_start(event_id: u32) {
    info!("plugin event {event_id}: Game is starting");
    // Game is starting (plugin can be loaded and kept alive through multiple game plays)
    // After this event, all functions of the API marked as 'in game only' can be called

    let plugin = get_plugin();

    let setup = plugin.get_active_view_setup();
    info!("Active view setup:");
    info!("  View mode: {:?}", setup.viewMode);

    let table = plugin.get_table_info();
    info!("Active table: {}", table.path);

    plugin.push_notification("Hello World", 5000);
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
