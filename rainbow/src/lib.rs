use log::info;
use vpinball_plugin_api::bindings::{OptionUnit, VPX_OPT_SHOW_TWEAK, VPX_OPT_SHOW_UI};
use vpinball_plugin_api::{plugin, Plugin, VPXApi};

struct RainbowPlugin {}

impl Plugin for RainbowPlugin {
    fn new() -> Self {
        RainbowPlugin {}
    }

    fn on_load(&mut self, api: &mut dyn VPXApi) {
        info!("Rainbow plugin loading");

        let red_blue = ["Red", "Blue"];
        let page_id = "rainbow"; // how should a plugin know it's id? Or maybe it shouldn't
        let opt = api.get_option(
            page_id,
            "color",
            VPX_OPT_SHOW_UI | VPX_OPT_SHOW_TWEAK,
            "Use red or blue",
            0.0,
            1.0,
            1.0,
            0.0,
            OptionUnit::None,
            &red_blue,
        ) as i32;
        info!("Rainbow plugin option: {}", opt);
    }

    fn on_unload(&mut self) {
        info!("Rainbow plugin unloading");
    }
}

plugin!(RainbowPlugin);

#[cfg(test)]
mod tests {
    use super::*;
    use vpinball_plugin_api::test::TEST_SESSION_ID;
    use vpinball_plugin_api::test::{TestMsgPluginAPI, TestVPXPluginAPI};

    #[ignore]
    #[test]
    fn test_plugin_load_unload() {
        let vpx_api = TestVPXPluginAPI::init();
        let mut msg_api = TestMsgPluginAPI::init(&vpx_api);
        let session_id = TEST_SESSION_ID;
        // FIXME: currently we can't set the pointer to the vpx api when a broadcast message is sent
        PluginLoad(session_id, &mut msg_api);

        PluginUnload();
    }
}
