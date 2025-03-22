use accessibility::{attribute::AXAttribute, ui_element::AXUIElement};
use accessibility_sys::{
    AXError, AXUIElementRef, AXUIElementSetAttributeValue, AXValueCreate, kAXErrorSuccess,
    kAXPositionAttribute, kAXSizeAttribute, kAXValueTypeCGPoint, kAXValueTypeCGSize,
};
use core_foundation::{
    base::{TCFType, ToVoid},
    dictionary::CFDictionary,
    string::CFString,
};
use core_foundation_sys::{
    dictionary::CFDictionaryRef,
    number::{CFNumberGetValue, CFNumberRef, kCFNumberSInt32Type},
    string::CFStringRef,
};
use core_graphics::display::{self, CGDisplay, CGPoint, CGRect, CGSize, CGWindowID};
use std::ffi::c_void;

// Private API that makes everything possible for mapping between the Accessibility API and
// CoreGraphics
unsafe extern "C" {
    pub fn _AXUIElementGetWindow(element: AXUIElementRef, out: *mut CGWindowID) -> AXError;
}

// slack: pid=83052 id=21247

fn main() {
    let wins = current_windows();
    println!("{wins:#?}");

    // let axwin = get_axwindow(83052, 21247).unwrap();
    // if let Err(err) = set_pos(&axwin, -1201.0, 80.0) {
    //     println!("{err}");
    // };

    // if let Err(err) = set_size(&axwin, 400.0, 200.0) {
    //     println!("{err}");
    // };
}

macro_rules! set_attr {
    ($axwin:expr, $val:expr, $ty:expr, $name:expr) => {
        unsafe {
            let val = AXValueCreate($ty, &mut $val as *mut _ as *mut c_void);
            let err = AXUIElementSetAttributeValue(
                $axwin.as_concrete_TypeRef(),
                CFString::new($name).as_concrete_TypeRef(),
                val as _,
            );

            if err == kAXErrorSuccess {
                Ok(())
            } else {
                Err(err)
            }
        }
    };
}

fn set_size(axwin: &AXUIElement, w: f64, h: f64) -> Result<(), AXError> {
    let mut s = CGSize::new(w, h);
    set_attr!(axwin, s, kAXValueTypeCGSize, kAXSizeAttribute)
}

fn set_pos(axwin: &AXUIElement, x: f64, y: f64) -> Result<(), AXError> {
    let mut p = CGPoint::new(x, y);
    set_attr!(axwin, p, kAXValueTypeCGPoint, kAXPositionAttribute)
}

fn get_axwindow(pid: i32, winid: u32) -> Result<AXUIElement, &'static str> {
    let attr = AXUIElement::application(pid)
        .attribute(&AXAttribute::windows())
        .map_err(|_| "Failed to get windows attr")?;

    for ax_window in attr.get_all_values().into_iter() {
        unsafe {
            let mut id: CGWindowID = 0;
            if _AXUIElementGetWindow(ax_window as AXUIElementRef, &mut id) == kAXErrorSuccess
                && id == winid
            {
                return Ok(AXUIElement::wrap_under_get_rule(
                    ax_window as AXUIElementRef,
                ));
            }
        }
    }

    Err("Window not found")
}

// kCGWindowAlpha = 1;
// kCGWindowBounds =     {
//     Height = 1107;
//     Width = 1200;
//     X = "-1200";
//     Y = 25;
// };
// kCGWindowIsOnscreen = 1;
// kCGWindowLayer = 0;
// kCGWindowMemoryUsage = 2176;
// kCGWindowNumber = 21247;
// kCGWindowOwnerName = Slack;
// kCGWindowOwnerPID = 83052;
// kCGWindowSharingState = 0;
// kCGWindowStoreType = 1;
#[derive(Debug, Clone)]
struct WinInfo {
    win_id: u32,
    owner_pid: i32,
    window_layer: i32, // do we only care about layer 0?
    bounds: CGRect,
    owner: String,
    window_name: Option<String>,
}

impl WinInfo {
    fn try_from_dict(dict: &CFDictionary) -> Option<Self> {
        fn get_string(dict: &CFDictionary, key: &str) -> Option<String> {
            dict.find(CFString::new(key).to_void()).map(|value| {
                unsafe { CFString::wrap_under_get_rule(*value as CFStringRef) }.to_string()
            })
        }

        fn get_i32(dict: &CFDictionary, key: &str) -> Option<i32> {
            let value = dict.find(CFString::new(key).to_void())?;
            let mut result = 0;
            unsafe {
                CFNumberGetValue(
                    *value as CFNumberRef,
                    kCFNumberSInt32Type,
                    (&mut result as *mut i32).cast(),
                )
            };

            Some(result)
        }

        fn get_dict(dict: &CFDictionary, key: &str) -> Option<CFDictionary> {
            let value = dict.find(CFString::new(key).to_void())?;
            Some(unsafe { CFDictionary::wrap_under_get_rule(*value as CFDictionaryRef) })
        }

        let win_id = get_i32(dict, "kCGWindowNumber")? as u32;
        let owner_pid = get_i32(dict, "kCGWindowOwnerPID")?;
        let window_layer = get_i32(dict, "kCGWindowLayer")?;
        let bounds = CGRect::from_dict_representation(&get_dict(dict, "kCGWindowBounds")?)?;
        let owner = get_string(dict, "kCGWindowOwnerName")?;
        let window_name = get_string(dict, "kCGWindowName");

        Some(Self {
            win_id,
            owner_pid,
            window_layer,
            bounds,
            owner,
            window_name,
        })
    }
}

fn current_windows() -> Option<Vec<WinInfo>> {
    let raw_infos = CGDisplay::window_list_info(
        display::kCGWindowListExcludeDesktopElements | display::kCGWindowListOptionOnScreenOnly,
        None,
    )?;
    let mut infos = Vec::new();

    for win_info in raw_infos.iter() {
        let dict = unsafe {
            CFDictionary::<*const c_void, *const c_void>::wrap_under_get_rule(
                *win_info as CFDictionaryRef,
            )
        };
        if let Some(info) = WinInfo::try_from_dict(&dict) {
            infos.push(info);
        }
    }

    Some(infos)
}
