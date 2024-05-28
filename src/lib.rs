#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

static mut vpx: *mut VPXPluginAPI = std::ptr::null_mut();

extern "C" fn onGameStart(eventId: u32, _data: *mut std::ffi::c_void) {
    println!("plugin event {eventId}: Game is starting");
    // Game is starting (plugin can be loaded and kept alive through multiple game plays)
    // After this event, all functions of the API marked as 'in game only' can be called
    let str = CString::new("Hello World").unwrap();
    unsafe {
        (*vpx).PushNotification.unwrap()(str.as_ptr(), 5000);
    }
}

extern "C" fn onGameEnd(eventId: u32, _data: *mut std::ffi::c_void) {
    println!("plugin event {eventId}: Game is ending");
    // Game is ending
    // After this event, all functions of the API marked as 'in game only' may not be called anymore
}

extern "C" fn onPrepareFrame(_eventId: u32, _data: *mut std::ffi::c_void) {
    //println!("plugin event {eventId}: Preparing frame");
    // Called when the player is about to prepare a new frame
    // This can be used to tweak any visual parameter before building the frame (for example head tracking,...)
}

#[no_mangle]
pub extern "C" fn PluginLoad(api: *mut VPXPluginAPI) {
    let event_id_start = CString::from_vec_with_nul(VPX_EVT_ON_GAME_START.to_vec()).unwrap();
    let event_id_end = CString::from_vec_with_nul(VPX_EVT_ON_GAME_END.to_vec()).unwrap();
    let event_id_prepare_frame =
        CString::from_vec_with_nul(VPX_EVT_ON_PREPARE_FRAME.to_vec()).unwrap();
    unsafe {
        vpx = api;

        (*vpx).SubscribeEvent.unwrap()(
            (*vpx).GetEventID.unwrap()(event_id_start.as_ptr()),
            Some(onGameStart),
        );

        (*vpx).SubscribeEvent.unwrap()(
            (*vpx).GetEventID.unwrap()(event_id_end.as_ptr()),
            Some(onGameEnd),
        );

        (*vpx).SubscribeEvent.unwrap()(
            (*vpx).GetEventID.unwrap()(event_id_prepare_frame.as_ptr()),
            Some(onPrepareFrame),
        );
    }
}

#[no_mangle]
pub extern "C" fn PluginUnload() {
    let event_id_start = CString::from_vec_with_nul(VPX_EVT_ON_GAME_START.to_vec()).unwrap();
    let event_id_end = CString::from_vec_with_nul(VPX_EVT_ON_GAME_END.to_vec()).unwrap();
    let event_id_prepare_frame =
        CString::from_vec_with_nul(VPX_EVT_ON_PREPARE_FRAME.to_vec()).unwrap();
    unsafe {
        (*vpx).UnsubscribeEvent.unwrap()(
            (*vpx).GetEventID.unwrap()(event_id_start.as_ptr()),
            Some(onGameStart),
        );
        (*vpx).UnsubscribeEvent.unwrap()(
            (*vpx).GetEventID.unwrap()(event_id_end.as_ptr()),
            Some(onGameEnd),
        );
        (*vpx).UnsubscribeEvent.unwrap()(
            (*vpx).GetEventID.unwrap()(event_id_prepare_frame.as_ptr()),
            Some(onPrepareFrame),
        );
        vpx = std::ptr::null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{c_uint, CStr};

    struct DummyVPXPluginAPI;
    impl DummyVPXPluginAPI {
        fn init() -> VPXPluginAPI {
            unsafe extern "C" fn dummy_subscribe_event(
                eventId: c_uint,
                _callback: vpxpi_event_callback,
            ) {
                println!("dummy subscribe_event({eventId})");
            }

            unsafe extern "C" fn dummy_unsubscribe_event(
                eventId: c_uint,
                _callback: vpxpi_event_callback,
            ) {
                println!("dummy unsubscribe_event({eventId})");
            }

            unsafe extern "C" fn dummy_get_event_id(
                event_name: *const std::os::raw::c_char,
            ) -> c_uint {
                let str_event_id = CStr::from_ptr(event_name).to_str().unwrap();
                let event_id: i32 = match str_event_id {
                    "VPX.OnGameStart" => 1,
                    "VPX.OnGameEnd" => 2,
                    "VPX.OnPrepareFrame" => 3,
                    _ => -1,
                };
                println!("dummy get_event_id(\"{str_event_id}\") -> {event_id}");
                event_id as c_uint
            }

            VPXPluginAPI {
                SubscribeEvent: Some(dummy_subscribe_event),
                UnsubscribeEvent: Some(dummy_unsubscribe_event),
                BroadcastEvent: None,
                GetTableInfo: None,
                GetEventID: Some(dummy_get_event_id),
                PushNotification: None,
                UpdateNotification: None,
                DisableStaticPrerendering: None,
                GetActiveViewSetup: None,
                GetOption: None,
                SetActiveViewSetup: None,
            }
        }
    }

    #[test]
    fn test_plugin_load_unload() {
        let mut api = DummyVPXPluginAPI::init();
        PluginLoad(&mut api);
        PluginUnload();
    }
}
