#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
#![allow(unused)]
#![allow(clippy::upper_case_acronyms)]

pub mod bindings;
pub mod test;

use log::{info, warn};
use std::collections::HashMap;
use std::ffi::{c_uint, CStr, CString};
use std::fmt::Debug;
use std::os::raw::{c_char, c_void};

// we redefine the constants here to avoid the need to translate from C to Rust
// MsgPlugin
pub const VPXPI_NAMESPACE: &str = cstr_to_str(bindings::VPXPI_NAMESPACE);
pub const VPXPI_MSG_GET_API: &str = cstr_to_str(bindings::VPXPI_MSG_GET_API);
pub const VPXPI_EVT_ON_GAME_START: &str = cstr_to_str(bindings::VPXPI_EVT_ON_GAME_START);
pub const VPXPI_EVT_ON_GAME_END: &str = cstr_to_str(bindings::VPXPI_EVT_ON_GAME_END);
pub const VPXPI_EVT_ON_PREPARE_FRAME: &str = cstr_to_str(bindings::VPXPI_EVT_ON_PREPARE_FRAME);
pub const VPXPI_EVT_ON_SETTINGS_CHANGED: &str =
    cstr_to_str(bindings::VPXPI_EVT_ON_SETTINGS_CHANGED);

// PinMamePlugin
pub const PMPI_NAMESPACE: &str = cstr_to_str(bindings::PMPI_NAMESPACE);
pub const PMPI_EVT_ON_GAME_START: &str = cstr_to_str(bindings::PMPI_EVT_ON_GAME_START);
pub const PMPI_EVT_ON_GAME_END: &str = cstr_to_str(bindings::PMPI_EVT_ON_GAME_END);

// CorePlugin
pub const CTLPI_NAMESPACE: &str = cstr_to_str(bindings::CTLPI_NAMESPACE);
pub const CTLPI_GETDMD_SRC_MSG: &str = cstr_to_str(bindings::CTLPI_GETDMD_SRC_MSG);
pub const CTLPI_GETDMD_RENDER_MSG: &str = cstr_to_str(bindings::CTLPI_GETDMD_RENDER_MSG);
pub const CTLPI_GETDMD_IDENTIFY_MSG: &str = cstr_to_str(bindings::CTLPI_GETDMD_IDENTIFY_MSG);

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
        unit: bindings::OptionUnit,
        values: &[&str],
    ) -> f32;

    fn push_notification(&self, message: &str, length_ms: u32);

    fn broadcast_msg(&self, endpoint_id: c_uint, msg_name_space: &str, msg_name: &str);

    fn get_active_view_setup(&self) -> bindings::VPXViewSetupDef;

    fn subscribe_msg(
        &mut self,
        msg_name_space: &str,
        msg_name: &str,
        callback_closure: Box<dyn Fn(u32)>,
    );
}

pub struct WrappedPluginApi {
    session_id: c_uint,
    msg: *mut bindings::MsgPluginAPI,
    vpx: *mut bindings::VPXPluginAPI,
    callbacks: HashMap<u32, *mut c_void>,
}

impl WrappedPluginApi {
    pub fn new(session_id: c_uint, msg: *mut bindings::MsgPluginAPI) -> Self {
        Self {
            session_id,
            msg,
            vpx: std::ptr::null_mut(),
            callbacks: HashMap::new(),
        }
    }
}

pub struct PluginWrapper<P: Plugin> {
    pub(crate) plugin: P,
    api: WrappedPluginApi,
}

impl<P: Plugin> PluginWrapper<P> {
    pub fn new(plugin: P, session_id: c_uint, msg: *mut bindings::MsgPluginAPI) -> Self {
        Self {
            plugin,
            api: WrappedPluginApi::new(session_id, msg),
        }
    }

    pub fn load(&mut self) {
        info!("load()");
        let vpxpi_name_space: *const c_char = VPXPI_NAMESPACE.as_ptr() as *const c_char;
        let vpxpi_get_api: *const c_char = VPXPI_MSG_GET_API.as_ptr() as *const c_char;
        unsafe {
            let msg_id = (*self.api.msg).GetMsgID.unwrap()(vpxpi_name_space, vpxpi_get_api);
            // sends the pointer location of the vpx api to the plugin system for populating the vpx pointer
            (*self.api.msg).BroadcastMsg.unwrap()(
                self.api.session_id,
                msg_id,
                &mut self.api.vpx as *mut *mut bindings::VPXPluginAPI as *mut c_void,
            );
        }
        self.plugin.on_load(&mut self.api);
    }

    pub fn unload(&mut self) {
        info!("unload()");
        self.plugin.on_unload();
        // unsubscribe all events
        for (event_id, callback) in self.api.callbacks.iter() {
            unsafe {
                info!("Unsubscribing for event_id {event_id}");
                (*self.api.msg).UnsubscribeMsg.unwrap()(*event_id, Some(trampoline));
                // free the callback
                drop(Box::from_raw(*callback as *mut Box<dyn Fn(u32)>));
            }
        }
        // TODO we have to call ReleaseMsgID for all messages we looked up with GetMsgID
        // self.api.msg.ReleaseMsgID.unwrap()(...);
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
            let mut table_info = bindings::VPXTableInfo {
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
        unit: bindings::OptionUnit,
        // array of strings
        values: &[&str],
    ) -> f32 {
        info!("get_option({option_name})");
        unsafe {
            let page_id = CString::new(page_id).unwrap();
            let option_id = CString::new(option_id).unwrap();
            let option_name = CString::new(option_name).unwrap();
            let raws = values
                .iter()
                .map(|s| CString::new(s.as_bytes()).unwrap().into_raw())
                .collect::<Vec<_>>();
            let values_ptr: *mut *const ::std::os::raw::c_char =
                raws.as_ptr() as *mut *const ::std::os::raw::c_char;

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
                values_ptr,
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

    fn broadcast_msg(&self, endpoint_id: c_uint, msg_name_space: &str, msg_name: &str) {
        info!("broadcast_event({endpoint_id}, {msg_name_space}, {msg_name})");
        let msg_name_space_c = CString::new(msg_name_space).unwrap();
        let msg_name_c = CString::new(msg_name).unwrap();
        let msg_id = unsafe {
            (*self.msg).GetMsgID.unwrap()(msg_name_space_c.as_ptr(), msg_name_c.as_ptr())
        };
        unsafe {
            (*self.msg).BroadcastMsg.unwrap()(endpoint_id, msg_id, std::ptr::null_mut());
        }
    }

    fn get_active_view_setup(&self) -> bindings::VPXViewSetupDef {
        info!("get_active_view_setup()");
        unsafe {
            // create a mutable pointer to a VPXPluginAPI_ViewSetupDef
            let mut view_setup = bindings::VPXViewSetupDef {
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

    fn subscribe_msg(
        &mut self,
        msg_name_space: &str,
        msg_name: &str,
        callback_closure: Box<dyn Fn(u32)>,
    ) {
        info!("subscribe_event({msg_name_space}, {msg_name})");
        let msg_name_space_c = CString::new(msg_name_space).unwrap();
        let msg_name_c = CString::new(msg_name).unwrap();
        let message_id = unsafe {
            (*self.msg).GetMsgID.unwrap()(msg_name_space_c.as_ptr(), msg_name_c.as_ptr())
        };
        // only allow one callback per event
        assert!(
            !self.callbacks.contains_key(&message_id),
            "Event {msg_name} already subscribed"
        );

        // Wrap it again in a Box to keep it alive.
        // Not sure why this is required, but otherwise we get 0x1 for trivial closures.
        // see https://users.rust-lang.org/t/how-to-convert-box-dyn-fn-into-raw-pointer-and-then-call-it/104410/2
        let wrapped = Box::new(callback_closure);
        let user_data: *mut c_void = Box::into_raw(wrapped) as *mut c_void;
        // can't be 0x1
        assert_ne!(user_data as u64, 0x1, "Invalid user_data");
        self.callbacks.insert(message_id, user_data);
        info!("Plugin: Subscribing for event_id {message_id} with user_data {user_data:?}");
        unsafe {
            (*self.msg).SubscribeMsg.unwrap()(
                self.session_id,
                message_id,
                Some(trampoline),
                user_data,
            );
        }
    }
}

pub trait Plugin: Sized {
    fn new() -> Self;
    fn on_load(&mut self, api: &mut dyn VPXApi);
    fn on_unload(&mut self);
}

pub struct VPXPlugin {
    vpx: *mut bindings::VPXPluginAPI,
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

const fn cstr_to_str(bytes: &[u8]) -> &str {
    match CStr::from_bytes_with_nul(bytes) {
        Ok(c) => match c.to_str() {
            Ok(s) => s,
            Err(_) => "[not_utf8]",
        },
        Err(_) => "[not_cstring]",
    }
}

#[macro_export]
macro_rules! plugin {
    ($plugin:ident) => {
        use vpinball_plugin_api::bindings::MsgPluginAPI;
        use vpinball_plugin_api::PluginWrapper;

        use std::ffi::c_uint;

        // TODO is this a good idea, how can we keep track of the instance?
        /// Everything should be called from a single thread that originates on the vpinball side.
        static mut PLUGIN: Option<std::rc::Rc<PluginWrapper<$plugin>>> = None;

        pub fn get_plugin_api() -> &'static dyn VPXApi {
            unsafe {
                match PLUGIN {
                    Some(ref wrapper_rc) => wrapper_rc.get_api(),
                    None => panic!("Plugin not loaded"),
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn PluginLoad(session_id: c_uint, msg: *mut MsgPluginAPI) {
            // TODO how does this work with multiple plugins?
            simple_logger::SimpleLogger::new().env().init().unwrap();
            // fail if already loaded
            assert!(unsafe { PLUGIN.is_none() }, "Plugin already loaded");
            log::info!("PluginLoad()");
            unsafe {
                let plugin = $plugin::new();
                // create a wrapper around the plugin
                let mut wrapper = PluginWrapper::new(plugin, session_id, msg);
                wrapper.load();
                PLUGIN = Some(std::rc::Rc::new(wrapper));
            }
        }

        #[no_mangle]
        pub extern "C" fn PluginUnload() {
            unsafe {
                if let Some(wrapper_rc) = PLUGIN.take() {
                    match std::rc::Rc::try_unwrap(wrapper_rc) {
                        Ok(mut wrapper) => {
                            log::info!("PluginUnload()");
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
