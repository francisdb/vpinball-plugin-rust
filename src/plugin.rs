#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
#![allow(unused)]
#![allow(clippy::upper_case_acronyms)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use log::info;
use simple_logger::init;
use std::collections::HashMap;
use std::ffi::{c_uint, CStr, CString};
use std::fmt::Debug;
use std::os::raw::{c_char, c_void};

pub trait VPXApi {
    fn get_table_info(&self) -> TableInfo;
    fn get_option(
        &self,
        page_id: &str,
        option_id: &str,
        show_mask: u32,
        option_name: &str,
        min_value: f32,
        max_value: f32,
        step: f32,
        default_value: f32,
        unit: OptionUnit,
        values: *mut *const ::std::os::raw::c_char,
    ) -> f32;

    fn push_notification(&self, message: &str, length_ms: u32);

    fn broadcast_event(&self, event_name: &str);

    fn get_active_view_setup(&self) -> VPXViewSetupDef;

    fn subscribe_event(&mut self, event_name: &str, callback_closure: Box<dyn Fn(u32)>);
}

pub struct WrappedPluginApi {
    vpx: *mut VPXPluginAPI,
    callbacks: HashMap<u32, *mut c_void>,
}

impl WrappedPluginApi {
    pub fn new(vpx: *mut VPXPluginAPI) -> Self {
        Self {
            vpx,
            callbacks: HashMap::new(),
        }
    }
}

pub(crate) struct PluginWrapper<P: Plugin> {
    pub(crate) plugin: P,
    api: WrappedPluginApi,
}

impl<P: Plugin> PluginWrapper<P> {
    pub fn new(plugin: P, vpx: *mut VPXPluginAPI) -> Self {
        Self {
            plugin,
            api: WrappedPluginApi::new(vpx),
        }
    }

    pub fn load(&mut self) {
        info!("load()");
        self.plugin.on_load(&mut self.api);
    }

    pub fn unload(&mut self) {
        info!("unload()");
        self.plugin.on_unload();
        // unsubscribe all events
        for (event_id, callback) in self.api.callbacks.iter() {
            unsafe {
                info!("Unsubscribing for event_id {event_id}");
                (*self.api.vpx).UnsubscribeEvent.unwrap()(*event_id, Some(trampoline));
                // free the callback
                drop(Box::from_raw(*callback as *mut Box<dyn Fn(u32)>));
            }
        }
        self.api.callbacks.clear();
        self.api.vpx = std::ptr::null_mut();
    }

    pub fn get_api(&self) -> &dyn VPXApi {
        &self.api
    }
}

impl VPXApi for WrappedPluginApi {
    fn get_table_info(&self) -> TableInfo {
        info!("get_table_info()");
        unsafe {
            // create a mutable pointer to a VPXPluginAPI_TableInfo
            let mut table_info = VPXTableInfo {
                path: std::ptr::null(),
                tableWidth: 0.0,
                tableHeight: 0.0,
            };
            (*self.vpx).GetTableInfo.unwrap()(&mut table_info);
            let path = CStr::from_ptr(table_info.path)
                .to_str()
                .unwrap()
                .to_string();
            // TODO how long does this table_info.path live?
            //   should we free it?
            TableInfo {
                path,
                tableWidth: table_info.tableWidth,
                tableHeight: table_info.tableHeight,
            }
        }
    }

    fn get_option(
        &self,
        page_id: &str,
        option_id: &str,
        show_mask: u32,
        option_name: &str,
        min_value: f32,
        max_value: f32,
        step: f32,
        default_value: f32,
        unit: OptionUnit,
        values: *mut *const ::std::os::raw::c_char,
    ) -> f32 {
        info!("get_option({option_name})");
        unsafe {
            let page_id = CString::new(page_id).unwrap();
            let option_id = CString::new(option_id).unwrap();
            let option_name = CString::new(option_name).unwrap();
            (*self.vpx).GetOption.unwrap()(
                page_id.as_ptr(),
                option_id.as_ptr(),
                show_mask,
                option_name.as_ptr(),
                min_value,
                max_value,
                step,
                default_value,
                unit.into(),
                values,
            )
        }
    }

    fn push_notification(&self, message: &str, length_ms: u32) {
        info!("push_notification({message}, {length_ms} ms)");
        let message_c = CString::new(message).unwrap();
        unsafe {
            (*self.vpx).PushNotification.unwrap()(message_c.as_ptr(), length_ms);
        }
    }

    fn broadcast_event(&self, event_name: &str) {
        info!("broadcast_event({event_name})");
        let event_id_c = CString::new(event_name).unwrap();
        let event_id = unsafe { (*self.vpx).GetEventID.unwrap()(event_id_c.as_ptr()) };
        unsafe {
            (*self.vpx).BroadcastEvent.unwrap()(event_id, std::ptr::null_mut());
        }
    }

    fn get_active_view_setup(&self) -> VPXViewSetupDef {
        info!("get_active_view_setup()");
        unsafe {
            // create a mutable pointer to a VPXPluginAPI_ViewSetupDef
            let mut view_setup = VPXViewSetupDef {
                viewMode: 0,
                sceneScaleX: 0.0,
                sceneScaleY: 0.0,
                sceneScaleZ: 0.0,
                viewX: 0.0,
                viewY: 0.0,
                viewZ: 0.0,
                lookAt: 0.0,
                viewportRotation: 0.0,
                FOV: 0.0,
                layback: 0.0,
                viewHOfs: 0.0,
                viewVOfs: 0.0,
                windowTopZOfs: 0.0,
                windowBottomZOfs: 0.0,
                screenWidth: 0.0,
                screenHeight: 0.0,
                screenInclination: 0.0,
                realToVirtualScale: 0.0,
                interpupillaryDistance: 0.0,
            };
            (*self.vpx).GetActiveViewSetup.unwrap()(&mut view_setup);
            view_setup
        }
    }

    fn subscribe_event(&mut self, event_name: &str, callback_closure: Box<dyn Fn(u32)>) {
        info!("subscribe_event({event_name})");
        let event_id_c = CString::new(event_name).unwrap();
        let event_id = unsafe { (*self.vpx).GetEventID.unwrap()(event_id_c.as_ptr()) };
        // only allow one callback per event
        assert!(
            !self.callbacks.contains_key(&event_id),
            "Event {event_name} already subscribed"
        );

        // Wrap it again in a Box to keep it alive.
        // Not sure why this is required, but otherwise we get 0x1 for trivial closures.
        // see https://users.rust-lang.org/t/how-to-convert-box-dyn-fn-into-raw-pointer-and-then-call-it/104410/2
        let wrapped = Box::new(callback_closure);
        let user_data: *mut c_void = Box::into_raw(wrapped) as *mut c_void;
        // can't be 0x1
        assert_ne!(user_data as u64, 0x1, "Invalid user_data");
        self.callbacks.insert(event_id, user_data);
        unsafe {
            println!("Plugin: Subscribing for event_id {event_id} with user_data {user_data:?}");
            (*self.vpx).SubscribeEvent.unwrap()(event_id, Some(trampoline), user_data);
        }
    }
}

pub trait Plugin: Sized {
    fn new() -> Self;
    fn on_load(&mut self, api: &mut dyn VPXApi);
    fn on_unload(&mut self);
}

pub struct VPXPlugin {
    vpx: *mut VPXPluginAPI,
    on_load: fn(&mut VPXPlugin) -> (),
    on_unload: fn(&mut VPXPlugin) -> (),
    //callbacks: HashMap<String, EventCallback>,
    callbacks: HashMap<u32, *mut c_void>,
}

// https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/
//
unsafe extern "C" fn trampoline(event_id: c_uint, user_data: *mut c_void, _data: *mut c_void) {
    //info!("Plugin: trampoline({event_id} {user_data:?})");
    let user_data = &mut *(user_data as *mut Box<dyn Fn(u32)>);
    user_data(event_id);
}

#[derive(Debug)]
pub struct TableInfo {
    pub path: String,
    pub tableWidth: f32,
    pub tableHeight: f32,
}

pub enum OptionUnit {
    None,
    Percent,
}

impl From<OptionUnit> for VPXPluginAPI_OptionUnit {
    fn from(unit: OptionUnit) -> Self {
        match unit {
            OptionUnit::None => VPXPluginAPI_OptionUnit_NONE,
            OptionUnit::Percent => VPXPluginAPI_OptionUnit_PERCENT,
        }
    }
}

pub const EVENT_ON_GAME_START: &str = "VPX.OnGameStart";
pub const EVENT_ON_GAME_END: &str = "VPX.OnGameEnd";
pub const EVENT_ON_PREPARE_FRAME: &str = "VPX.OnPrepareFrame";
pub const EVENT_ON_SETTINGS_CHANGED: &str = "VPX.OnSettingsChanged";

#[macro_export]
macro_rules! plugin {
    ($plugin:ident) => {
        use plugin::PluginWrapper;

        // TODO is this a good idea, how can we keep track of the instance?
        /// Everything should be called from a single thread that originates on the vpinball side.
        static mut PLUGIN: Option<Rc<PluginWrapper<$plugin>>> = None;

        pub fn get_plugin_api() -> &'static dyn VPXApi {
            unsafe {
                match PLUGIN {
                    Some(ref wrapper_rc) => wrapper_rc.get_api(),
                    None => panic!("Plugin not loaded"),
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn PluginLoad(vpx: *mut VPXPluginAPI) {
            simple_logger::SimpleLogger::new().env().init().unwrap();
            // fail if already loaded
            assert!(unsafe { PLUGIN.is_none() }, "Plugin already loaded");
            info!("PluginLoad()");
            unsafe {
                let plugin = $plugin::new();
                // create a wrapper around the plugin
                let mut wrapper = PluginWrapper::new(plugin, vpx);
                wrapper.load();
                PLUGIN = Some(Rc::new(wrapper));
            }
        }

        #[no_mangle]
        pub extern "C" fn PluginUnload() {
            unsafe {
                if let Some(wrapper_rc) = PLUGIN.take() {
                    match Rc::try_unwrap(wrapper_rc) {
                        Ok(mut wrapper) => {
                            info!("PluginUnload()");
                            wrapper.unload();
                        }
                        Err(_) => {
                            panic!("Failed to get mutable reference to plugin");
                        }
                    }
                }
            }
        }
    };
}

#[cfg(test)]
pub mod tests {
    use crate::plugin::{vpxpi_event_callback, VPXPluginAPI};
    use std::ffi::{c_uint, CStr};

    pub struct TestVPXPluginAPI;
    impl TestVPXPluginAPI {
        pub fn init() -> VPXPluginAPI {
            unsafe extern "C" fn subscribe_event(
                event_id: c_uint,
                _callback: vpxpi_event_callback,
                _user_data: *mut std::ffi::c_void,
            ) {
                println!("TestVPXPluginAPI::subscribe_event({event_id})");
            }

            unsafe extern "C" fn unsubscribe_event(
                event_id: c_uint,
                _callback: vpxpi_event_callback,
            ) {
                println!("TestVPXPluginAPI::unsubscribe_event({event_id})");
            }

            unsafe extern "C" fn get_event_id(event_name: *const std::os::raw::c_char) -> c_uint {
                let str_event_id = CStr::from_ptr(event_name).to_str().unwrap();
                let event_id: i32 = match str_event_id {
                    "VPX.OnGameStart" => 1,
                    "VPX.OnGameEnd" => 2,
                    "VPX.OnPrepareFrame" => 3,
                    _ => -1,
                };
                println!("TestVPXPluginAPI::get_event_id(\"{str_event_id}\") -> {event_id}");
                event_id as c_uint
            }

            VPXPluginAPI {
                SubscribeEvent: Some(subscribe_event),
                UnsubscribeEvent: Some(unsubscribe_event),
                BroadcastEvent: None,
                GetTableInfo: None,
                GetEventID: Some(get_event_id),
                PushNotification: None,
                UpdateNotification: None,
                DisableStaticPrerendering: None,
                GetActiveViewSetup: None,
                GetOption: None,
                SetActiveViewSetup: None,
            }
        }
    }
}
