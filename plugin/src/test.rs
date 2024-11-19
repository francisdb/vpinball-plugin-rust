use crate::bindings::msgpi_msg_callback;
use crate::bindings::msgpi_timer_callback;
use crate::bindings::MsgPluginAPI;
use log::{info, warn};
use std::ffi::{c_uint, CStr};

pub const TEST_SESSION_ID: c_uint = 123;

pub struct TestVPXPluginAPI;
impl TestVPXPluginAPI {
    pub fn init() -> MsgPluginAPI {
        unsafe extern "C" fn subscribe_msg(
            endpoint_id: c_uint,
            msg_id: c_uint,
            _callback: msgpi_msg_callback,
            _user_data: *mut std::ffi::c_void,
        ) {
            info!("TestVPXPluginAPI::subscribe_msg({msg_id})");
        }

        unsafe extern "C" fn unsubscribe_msg(msg_id: c_uint, _callback: msgpi_msg_callback) {
            info!("TestVPXPluginAPI::unsubscribe_msg({msg_id})");
        }

        unsafe extern "C" fn get_msg_id(
            name_space: *const std::os::raw::c_char,
            name: *const std::os::raw::c_char,
        ) -> c_uint {
            let str_name_space = CStr::from_ptr(name_space).to_str().unwrap();
            let str_name = CStr::from_ptr(name).to_str().unwrap();
            let event_id: i32 = match (str_name_space, str_name) {
                ("VPX", "OnGameStart") => 1,
                ("VPX", "OnGameEnd") => 2,
                ("VPX", "OnPrepareFrame") => 3,
                ("VPX", "OnSettingsChanged") => 4,
                ("VPX", "GetAPI") => 5,
                _ => unimplemented!("Unknown event {str_name_space}:{str_name}"),
            };
            info!(
                "TestVPXPluginAPI::get_msg_id(\"{str_name_space}\" ,\"{str_name}\") -> {event_id}"
            );
            event_id as c_uint
        }

        unsafe extern "C" fn broadcast_msg(
            endpoint_id: c_uint,
            msg_id: c_uint,
            data: *mut std::ffi::c_void,
        ) {
            assert_eq!(endpoint_id, TEST_SESSION_ID);
            info!("TestVPXPluginAPI::broadcast_msg({endpoint_id}, {msg_id}, {data:?})");
            // TODO if the vpx interface is requested we should set the pointer
            if msg_id == 5 {
                warn!("Requesting VPXPluginAPI pointer not implemented");
            }
        }

        unsafe extern "C" fn release_msg_id(msg_id: c_uint) {
            info!("TestVPXPluginAPI::release_msg_id({msg_id})");
        }

        unsafe extern "C" fn get_settings(
            name_space: *const ::std::os::raw::c_char,
            name: *const ::std::os::raw::c_char,
            valueBuf: *mut ::std::os::raw::c_char,
            valueBufSize: ::std::os::raw::c_uint,
        ) {
            info!("TestVPXPluginAPI::get_settings() not implemented");
        }

        unsafe extern "C" fn run_on_main_thread(
            delayInS: f64,
            callback: msgpi_timer_callback,
            userData: *mut ::std::os::raw::c_void,
        ) {
            info!("TestVPXPluginAPI::run_on_main_thread() not implemented");
        }

        MsgPluginAPI {
            SubscribeMsg: Some(subscribe_msg),
            UnsubscribeMsg: Some(unsubscribe_msg),
            GetMsgID: Some(get_msg_id),
            BroadcastMsg: Some(broadcast_msg),
            ReleaseMsgID: Some(release_msg_id),
            GetSetting: Some(get_settings),
            RunOnMainThread: Some(run_on_main_thread),
        }
    }
}
