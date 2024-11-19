use log::info;
use vpinball_plugin_api::{plugin, Plugin, VPXApi};

struct RainbowPlugin {}

impl Plugin for RainbowPlugin {
    fn new() -> Self {
        RainbowPlugin {}
    }

    fn on_load(&mut self, _api: &mut dyn VPXApi) {
        info!("Rainbow plugin loading");
    }

    fn on_unload(&mut self) {
        info!("Rainbow plugin unloading");
    }
}

plugin!(RainbowPlugin);

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
