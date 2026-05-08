use std::collections::HashMap;
use std::ffi::CStr;
use std::process::Command;
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};

use libc::c_void;

use crate::caps::is_caps_lock_on;
use crate::ffi;

#[derive(Clone, Copy)]
struct LedElement {
    device: ffi::IOHIDDeviceRef,
    element: ffi::IOHIDElementRef,
}

pub struct CapsLockLedManager {
    manager: Option<ffi::IOHIDManagerRef>,
    led_elements: Vec<LedElement>,
    device_cooldowns: HashMap<usize, Instant>,
    silent: bool,
}

impl CapsLockLedManager {
    pub fn new(silent: bool) -> Result<Self, String> {
        let mut manager = Self {
            manager: None,
            led_elements: Vec::new(),
            device_cooldowns: HashMap::new(),
            silent,
        };
        manager.create_manager()?;
        Ok(manager)
    }

    pub fn blink(&mut self, times: usize, interval: Duration) {
        let caps_on = is_caps_lock_on();
        for _ in 0..times {
            if caps_on {
                self.toggle(false);
                thread::sleep(interval);
                self.toggle(true);
            } else {
                self.toggle(true);
                thread::sleep(interval);
                self.toggle(false);
            }
            thread::sleep(interval);
        }
    }

    fn toggle(&mut self, on: bool) {
        let now = Instant::now();
        let mut should_reinitialize = false;

        for led in self.led_elements.iter().copied() {
            let device_key = led.device as usize;
            if self
                .device_cooldowns
                .get(&device_key)
                .is_some_and(|cooldown_until| now < *cooldown_until)
            {
                continue;
            }

            let result = unsafe { set_led_value(led.device, led.element, on) };
            match result {
                ffi::K_IORETURN_SUCCESS => {}
                ffi::K_IORETURN_ABORTED => return,
                ffi::K_IORETURN_EXCLUSIVE_ACCESS => {
                    self.device_cooldowns
                        .insert(device_key, now + Duration::from_secs(1));
                }
                ffi::K_KARABINER_SUCCESS_SKIP | ffi::K_IORETURN_NOT_OPEN => {
                    should_reinitialize = true;
                    break;
                }
                _ => {}
            }
        }

        if should_reinitialize {
            if !self.silent {
                println!("Re-initializing IOHIDManager...");
            }
            let _ = self.reinitialize();
        }
    }

    fn reinitialize(&mut self) -> Result<(), String> {
        self.device_cooldowns.clear();
        self.close_manager();
        self.create_manager()
    }

    fn create_manager(&mut self) -> Result<(), String> {
        let manager = unsafe { ffi::IOHIDManagerCreate(ptr::null(), ffi::K_IO_OPTIONS_TYPE_NONE) };
        if manager.is_null() {
            return Ok(());
        }

        if let Some(matching_array) = unsafe { create_keyboard_matching_array() } {
            unsafe {
                ffi::IOHIDManagerSetDeviceMatchingMultiple(manager, matching_array);
                ffi::CFRelease(matching_array);
            }
        }

        let open_result = unsafe { ffi::IOHIDManagerOpen(manager, ffi::K_IO_OPTIONS_TYPE_NONE) };
        if open_result == ffi::K_IORETURN_NOT_PERMITTED {
            unsafe {
                ffi::CFRelease(manager);
            }
            open_input_monitoring_preferences();
            return Err("Input Monitoring permissions need to be granted, exiting...".to_owned());
        }

        if open_result != ffi::K_IORETURN_SUCCESS {
            unsafe {
                ffi::CFRelease(manager);
            }
            if !self.silent {
                eprintln!("Unable to open IOHIDManager: {open_result}");
            }
            return Ok(());
        }

        self.manager = Some(manager);
        self.cache_devices();
        Ok(())
    }

    fn cache_devices(&mut self) {
        self.clear_led_cache();

        let Some(manager) = self.manager else {
            return;
        };

        let devices = unsafe { ffi::IOHIDManagerCopyDevices(manager) };
        if devices.is_null() {
            return;
        }

        let count = unsafe { ffi::CFSetGetCount(devices) };
        if count <= 0 {
            unsafe {
                ffi::CFRelease(devices);
            }
            return;
        }

        let mut values = vec![ptr::null(); count as usize];
        unsafe {
            ffi::CFSetGetValues(devices, values.as_mut_ptr());
        }

        for device_value in values {
            let device = device_value as ffi::IOHIDDeviceRef;
            if device.is_null() {
                continue;
            }

            if let Some(element) = unsafe { find_caps_lock_led(device) } {
                unsafe {
                    ffi::CFRetain(device);
                }
                self.led_elements.push(LedElement { device, element });
            }
        }

        unsafe {
            ffi::CFRelease(devices);
        }
    }

    fn close_manager(&mut self) {
        self.clear_led_cache();
        if let Some(manager) = self.manager.take() {
            unsafe {
                let _ = ffi::IOHIDManagerClose(manager, ffi::K_IO_OPTIONS_TYPE_NONE);
                ffi::CFRelease(manager);
            }
        }
    }

    fn clear_led_cache(&mut self) {
        for led in self.led_elements.drain(..) {
            unsafe {
                ffi::CFRelease(led.element);
                ffi::CFRelease(led.device);
            }
        }
    }
}

impl Drop for CapsLockLedManager {
    fn drop(&mut self) {
        self.close_manager();
    }
}

unsafe fn set_led_value(
    device: ffi::IOHIDDeviceRef,
    element: ffi::IOHIDElementRef,
    on: bool,
) -> ffi::IOReturn {
    let value = unsafe {
        ffi::IOHIDValueCreateWithIntegerValue(
            ptr::null(),
            element,
            ffi::mach_absolute_time(),
            if on { 1 } else { 0 },
        )
    };
    if value.is_null() {
        return ffi::K_IORETURN_NOT_OPEN;
    }

    let result = unsafe { ffi::IOHIDDeviceSetValue(device, element, value) };
    unsafe {
        ffi::CFRelease(value);
    }
    result
}

unsafe fn find_caps_lock_led(device: ffi::IOHIDDeviceRef) -> Option<ffi::IOHIDElementRef> {
    let usage_page_key = unsafe { create_cf_string(c"UsagePage") }?;
    let usage_page = unsafe { create_cf_number(ffi::K_HID_PAGE_LEDS) }?;
    let keys = [usage_page_key];
    let values = [usage_page];
    let matching = unsafe { create_cf_dictionary(&keys, &values)? };
    let elements = unsafe {
        ffi::IOHIDDeviceCopyMatchingElements(device, matching, ffi::K_IO_OPTIONS_TYPE_NONE)
    };

    unsafe {
        ffi::CFRelease(matching);
        ffi::CFRelease(usage_page_key);
        ffi::CFRelease(usage_page);
    }

    if elements.is_null() {
        return None;
    }

    let count = unsafe { ffi::CFArrayGetCount(elements) };
    let mut caps_lock_element = None;
    for index in 0..count {
        let element =
            unsafe { ffi::CFArrayGetValueAtIndex(elements, index) as ffi::IOHIDElementRef };
        if element.is_null() {
            continue;
        }

        let usage_page = unsafe { ffi::IOHIDElementGetUsagePage(element) };
        let usage = unsafe { ffi::IOHIDElementGetUsage(element) };
        if usage_page == ffi::K_HID_PAGE_LEDS as u32 && usage == ffi::K_HID_USAGE_LED_CAPS_LOCK {
            caps_lock_element = Some(unsafe { ffi::CFRetain(element) as ffi::IOHIDElementRef });
            break;
        }
    }

    unsafe {
        ffi::CFRelease(elements);
    }
    caps_lock_element
}

unsafe fn create_keyboard_matching_array() -> Option<ffi::CFArrayRef> {
    let usage_page_key = unsafe { create_cf_string(c"DeviceUsagePage") }?;
    let usage_key = unsafe { create_cf_string(c"DeviceUsage") }?;
    let usage_page = unsafe { create_cf_number(ffi::K_HID_PAGE_GENERIC_DESKTOP) }?;
    let usage = unsafe { create_cf_number(ffi::K_HID_USAGE_GD_KEYBOARD) }?;
    let keys = [usage_page_key, usage_key];
    let values = [usage_page, usage];
    let dictionary = unsafe { create_cf_dictionary(&keys, &values) };

    unsafe {
        ffi::CFRelease(usage_page_key);
        ffi::CFRelease(usage_key);
        ffi::CFRelease(usage_page);
        ffi::CFRelease(usage);
    }

    let dictionary = dictionary?;
    let array_values = [dictionary];
    let array = unsafe {
        ffi::CFArrayCreate(
            ptr::null(),
            array_values.as_ptr(),
            array_values.len() as ffi::CFIndex,
            &ffi::kCFTypeArrayCallBacks,
        )
    };

    unsafe {
        ffi::CFRelease(dictionary);
    }

    if array.is_null() { None } else { Some(array) }
}

unsafe fn create_cf_number(value: i32) -> Option<ffi::CFNumberRef> {
    let number = unsafe {
        ffi::CFNumberCreate(
            ptr::null(),
            ffi::K_CF_NUMBER_SINT32_TYPE,
            &value as *const i32 as *const c_void,
        )
    };
    if number.is_null() { None } else { Some(number) }
}

unsafe fn create_cf_string(value: &CStr) -> Option<ffi::CFStringRef> {
    let string = unsafe {
        ffi::CFStringCreateWithCString(ptr::null(), value.as_ptr(), ffi::K_CF_STRING_ENCODING_UTF8)
    };
    if string.is_null() { None } else { Some(string) }
}

unsafe fn create_cf_dictionary(
    keys: &[ffi::CFTypeRef],
    values: &[ffi::CFTypeRef],
) -> Option<ffi::CFDictionaryRef> {
    debug_assert_eq!(keys.len(), values.len());
    let dictionary = unsafe {
        ffi::CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            keys.len() as ffi::CFIndex,
            &ffi::kCFTypeDictionaryKeyCallBacks,
            &ffi::kCFTypeDictionaryValueCallBacks,
        )
    };
    if dictionary.is_null() {
        None
    } else {
        Some(dictionary)
    }
}

fn open_input_monitoring_preferences() {
    let _ = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
        .status();
}
