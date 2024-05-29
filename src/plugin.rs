#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::collections::HashMap;
use std::ffi::{c_uint, CStr, CString};
use std::fmt::{Debug, Display};
use std::os::raw::c_void;
use std::sync::Arc;

static mut PLUGIN: Option<Arc<VPXPlugin>> = None;

pub fn get_plugin() -> Arc<VPXPlugin> {
    unsafe { PLUGIN.as_ref().expect("Plugin not loaded").clone() }
}

unsafe extern "C" fn event_callback(event_id: c_uint, data: *mut c_void) {
    // go through a statically registered callback back to this instance
    get_plugin().callback(event_id, data);
}

pub struct VPXPlugin {
    vpx: *mut VPXPluginAPI,
    on_load: fn(&mut VPXPlugin) -> (),
    on_unload: fn(&mut VPXPlugin) -> (),
    //callbacks: HashMap<String, EventCallback>,
    callbacks: HashMap<u32, Box<dyn Fn(&VPXPlugin, u32)>>,
}

// TODO If on the vpinball side we could pass a *user_data to the callback
//   and registration functions we would be able to use this trampoline
// https://adventures.michaelfbryan.com/posts/rust-closures-in-ffi/
//
// unsafe extern "C" fn trampoline<F>(event_id: c_uint, user_data: *mut c_void)
// where
//     F: FnMut(c_uint),
// {
//     let user_data = &mut *(user_data as *mut F);
//     user_data(event_id);
// }
//
// pub fn get_trampoline<F>(_closure: &F) -> EventCallback
// where
//     F: FnMut(c_uint),
// {
//     trampoline::<F>
// }

type PluginCallback = fn(&mut VPXPlugin);

pub fn DoPluginLoad(api: *mut VPXPluginAPI, on_load: PluginCallback, on_unload: PluginCallback) {
    let mut plugin = VPXPlugin::new(on_load, on_unload);
    plugin.load(api);
    unsafe {
        PLUGIN = Some(Arc::new(plugin));
    }
}

pub fn DoPluginUnload() {
    unsafe {
        if let Some(plugin) = PLUGIN.take() {
            match Arc::try_unwrap(plugin) {
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

#[derive(Debug)]
pub struct TableInfo {
    pub path: String,
    pub tableWidth: f32,
    pub tableHeight: f32,
}

impl VPXPlugin {
    pub fn new(on_load: fn(&mut VPXPlugin) -> (), on_unload: fn(&mut VPXPlugin) -> ()) -> Self {
        VPXPlugin {
            vpx: std::ptr::null_mut(),
            on_load,
            on_unload,
            callbacks: HashMap::new(),
        }
    }

    pub fn load(&mut self, api: *mut VPXPluginAPI) {
        println!("load({:?})", api);
        // fail if already loaded
        assert!(self.vpx.is_null(), "Plugin already loaded");
        self.vpx = api;
        (self.on_load)(self);
    }

    pub fn unload(&mut self) {
        println!("unload()");
        // fail if not loaded
        assert!(!self.vpx.is_null(), "Plugin not loaded");
        (self.on_unload)(self);
        self.vpx = std::ptr::null_mut();
    }

    pub fn push_notification(&self, message: &str, length_ms: u32) {
        println!("push_notification({message}, {length_ms} ms)");
        let message_c = CString::new(message).unwrap();
        unsafe {
            (*self.vpx).PushNotification.unwrap()(message_c.as_ptr(), length_ms);
        }
    }

    pub fn broadcast_event(&self, event_name: &str) {
        println!("broadcast_event({event_name})");
        let event_id_c = CString::new(event_name).unwrap();
        let event_id = unsafe { (*self.vpx).GetEventID.unwrap()(event_id_c.as_ptr()) };
        unsafe {
            (*self.vpx).BroadcastEvent.unwrap()(event_id, std::ptr::null_mut());
        }
    }

    pub fn get_table_info(&self) -> TableInfo {
        println!("get_table_info()");
        unsafe {
            // create a mutable pointer to a VPXPluginAPI_TableInfo
            let mut table_info = VPXPluginAPI_TableInfo {
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

    pub fn get_active_view_setup(&self) -> VPXPluginAPI_ViewSetupDef {
        println!("get_active_view_setup()");
        unsafe {
            // create a mutable pointer to a VPXPluginAPI_ViewSetupDef
            let mut view_setup = VPXPluginAPI_ViewSetupDef {
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

    fn callback(&self, event_id: u32, _data: *mut c_void) {
        // if we have a callback registered for this event_id, call it
        if let Some(callback) = self.callbacks.get(&event_id) {
            callback(&self, event_id);
        }
    }

    pub fn subscribe_event(&mut self, event_name: &str, callback: Box<dyn Fn(&VPXPlugin, u32)>) {
        println!("subscribe_event({event_name})");
        let event_id_c = CString::new(event_name).unwrap();
        let event_id = unsafe { (*self.vpx).GetEventID.unwrap()(event_id_c.as_ptr()) };

        // self.callbacks.insert(event_name.to_string(), trampoline);
        // unsafe {
        //     (*self.vpx).SubscribeEvent.unwrap()(event_id, Some(trampoline));
        // }
        self.callbacks.insert(event_id, callback);
        unsafe {
            (*self.vpx).SubscribeEvent.unwrap()(event_id, Some(event_callback));
        }
    }

    pub fn unsubscribe_event(&mut self, event_name: &str) {
        println!("unsubscribe_event({event_name})");
        let event_id_c = CString::new(event_name).unwrap();
        let event_id = unsafe { (*self.vpx).GetEventID.unwrap()(event_id_c.as_ptr()) };
        // let callback = self.callbacks.remove(event_name).unwrap();
        // unsafe {
        //     (*self.vpx).UnsubscribeEvent.unwrap()(event_id, Some(callback));
        // }
        unsafe {
            (*self.vpx).UnsubscribeEvent.unwrap()(event_id, Some(event_callback));
        }
    }
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
