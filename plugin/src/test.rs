use crate::bindings::MsgPluginAPI;
use crate::bindings::VPXPluginAPI;
use crate::bindings::{msgpi_msg_callback, VPXTableInfo};
use crate::bindings::{msgpi_timer_callback, VPPluginAPI_OptionUnit};
use log::{info, warn};
use std::ffi::{c_uint, CStr};

pub const TEST_SESSION_ID: c_uint = 123;

pub struct TestVPXPluginAPI;

impl TestVPXPluginAPI {
    pub fn init() -> VPXPluginAPI {
        unsafe extern "C" fn get_table_info(info: *mut VPXTableInfo) {
            info!("TestVPXPluginAPI::get_table_info()");
        }

        unsafe extern "C" fn get_option(
            pageId: *const ::std::os::raw::c_char,
            optionId: *const ::std::os::raw::c_char,
            showMask: ::std::os::raw::c_uint,
            optionName: *const ::std::os::raw::c_char,
            minValue: f32,
            maxValue: f32,
            step: f32,
            defaultValue: f32,
            unit: VPPluginAPI_OptionUnit,
            values: *mut *const ::std::os::raw::c_char,
        ) -> f32 {
            info!("TestVPXPluginAPI::get_option()");
            0.0
        }

        VPXPluginAPI {
            GetTableInfo: Some(get_table_info),
            GetOption: Some(get_option),
            PushNotification: None,
            UpdateNotification: None,
            DisableStaticPrerendering: None,
            GetActiveViewSetup: None,
            SetActiveViewSetup: None,
        }
    }
}

pub struct TestMsgPluginAPI;

impl TestMsgPluginAPI {
    pub fn init(vpx_api: &VPXPluginAPI) -> MsgPluginAPI {
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
            let event_id: i32 = message_id_for(str_name_space, str_name);
            info!(
                "TestVPXPluginAPI::get_msg_id(\"{str_name_space}\" ,\"{str_name}\") -> {event_id}"
            );
            event_id as c_uint
        }

        fn message_id_for(str_name_space: &str, str_name: &str) -> i32 {
            match (str_name_space, str_name) {
                ("VPX", "OnGameStart") => 1,
                ("VPX", "OnGameEnd") => 2,
                ("VPX", "OnPrepareFrame") => 3,
                ("VPX", "OnSettingsChanged") => 4,
                ("VPX", "GetAPI") => 5,
                _ => unimplemented!("Unknown event {str_name_space}:{str_name}"),
            }
        }

        fn message_name_for(msg_id: c_uint) -> (&'static str, &'static str) {
            match msg_id {
                1 => ("VPX", "OnGameStart"),
                2 => ("VPX", "OnGameEnd"),
                3 => ("VPX", "OnPrepareFrame"),
                4 => ("VPX", "OnSettingsChanged"),
                5 => ("VPX", "GetAPI"),
                _ => unimplemented!("Unknown event {msg_id}"),
            }
        }

        unsafe extern "C" fn broadcast_msg(
            endpoint_id: c_uint,
            msg_id: c_uint,
            data: *mut std::ffi::c_void,
        ) {
            assert_eq!(endpoint_id, TEST_SESSION_ID);
            let (str_name_space, str_name) = message_name_for(msg_id);
            info!("TestVPXPluginAPI::broadcast_msg({endpoint_id}, {msg_id} ({str_name_space}:{str_name}))");
            // TODO if the vpx interface is requested we should set the pointer
            if msg_id == 5 {
                warn!("Requesting VPXPluginAPI pointer not implemented");
                //data = vpx_api.as_ptr() as *mut std::ffi::c_void;
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
