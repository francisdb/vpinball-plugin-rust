/// Our example plugin for Virtual Pinball
mod fpscounter;

use log::info;
use std::cell::RefCell;
use std::rc::Rc;

use vpinball_plugin_api::{
    plugin, Plugin, VPXApi, VPXPI_EVT_ON_GAME_END, VPXPI_EVT_ON_GAME_START,
    VPXPI_EVT_ON_PREPARE_FRAME, VPXPI_EVT_ON_SETTINGS_CHANGED, VPXPI_NAMESPACE,
};

struct FpsPlugin {
    fps_counter: Rc<RefCell<fpscounter::FPSCounter>>,
}

impl Plugin for FpsPlugin {
    fn new() -> Self {
        Self {
            fps_counter: Rc::new(RefCell::new(fpscounter::FPSCounter::new())),
        }
    }

    fn on_load(&mut self, vpx: &mut dyn VPXApi) {
        info!("Plugin loading");
        let fps_counter_clone = Rc::clone(&self.fps_counter);
        // TODO on the example this is the session_id that is passed on plugin
        vpx.subscribe_msg(
            VPXPI_NAMESPACE,
            VPXPI_EVT_ON_GAME_START,
            Box::new(|event_id| {
                info!("plugin event {event_id}: Game is starting");
                // Game is starting (plugin can be loaded and kept alive through multiple game plays)
                // After this event, all functions of the API marked as 'in game only' can be called

                let plugin = get_plugin_api();

                let setup = plugin.get_active_view_setup();
                info!("Active view setup:");
                info!("  View mode: {:?}", setup.viewMode);

                let table = plugin.get_table_info();
                info!("Active table: {}", table.path);

                plugin.push_notification("Hello World", 5000);
            }),
        );
        vpx.subscribe_msg(
            VPXPI_NAMESPACE,
            VPXPI_EVT_ON_GAME_END,
            Box::new(|event_id| {
                info!("plugin event {event_id}: Game is ending");
            }),
        );
        vpx.subscribe_msg(
            VPXPI_NAMESPACE,
            VPXPI_EVT_ON_PREPARE_FRAME,
            Box::new(move |_event_id| {
                let mut fps_counter = fps_counter_clone.borrow_mut();
                let fps = fps_counter.update();
                if let Some(fps) = fps {
                    info!("FPS: {:.2}", fps);
                }
            }),
        );
        vpx.subscribe_msg(
            VPXPI_NAMESPACE,
            VPXPI_EVT_ON_SETTINGS_CHANGED,
            Box::new(|_event_id| {
                info!("Settings changed");
            }),
        );
    }

    fn on_unload(&mut self) {
        info!("Plugin unloading");
    }
}

plugin!(FpsPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    use vpinball_plugin_api::test::TestVPXPluginAPI;
    use vpinball_plugin_api::test::TEST_SESSION_ID;

    #[test]
    fn test_plugin_load_unload() {
        let mut api = TestVPXPluginAPI::init();
        let session_id = TEST_SESSION_ID;
        PluginLoad(session_id, &mut api);

        PluginUnload();
    }
}
