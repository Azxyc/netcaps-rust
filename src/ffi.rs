#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use libc::{c_char, c_int, c_long, c_uint, c_void};

pub type CFAllocatorRef = *const c_void;
pub type CFArrayRef = *const c_void;
pub type CFDictionaryRef = *const c_void;
pub type CFIndex = c_long;
pub type CFNumberRef = *const c_void;
pub type CFNumberType = CFIndex;
pub type CFSetRef = *const c_void;
pub type CFStringRef = *const c_void;
pub type CFStringEncoding = u32;
pub type CFTypeRef = *const c_void;
pub type CGEventSourceStateID = c_uint;
pub type CGKeyCode = u16;
pub type IOHIDDeviceRef = *const c_void;
pub type IOHIDElementRef = *const c_void;
pub type IOHIDManagerRef = *const c_void;
pub type IOHIDValueRef = *const c_void;
pub type IOOptionBits = u32;
pub type IOReturn = c_int;

pub const K_CF_NUMBER_SINT32_TYPE: CFNumberType = 3;
pub const K_CF_STRING_ENCODING_UTF8: CFStringEncoding = 0x0800_0100;
pub const K_CG_EVENT_SOURCE_STATE_COMBINED_SESSION_STATE: CGEventSourceStateID = 0;
pub const K_HID_PAGE_GENERIC_DESKTOP: i32 = 0x01;
pub const K_HID_USAGE_GD_KEYBOARD: i32 = 0x06;
pub const K_HID_PAGE_LEDS: i32 = 0x08;
pub const K_HID_USAGE_LED_CAPS_LOCK: u32 = 0x02;
pub const K_IO_OPTIONS_TYPE_NONE: IOOptionBits = 0;
pub const K_IORETURN_SUCCESS: IOReturn = 0;
pub const K_IORETURN_EXCLUSIVE_ACCESS: IOReturn = -536_870_203;
pub const K_IORETURN_NOT_OPEN: IOReturn = -536_870_195;
pub const K_IORETURN_NOT_PERMITTED: IOReturn = -536_870_174;
pub const K_IORETURN_ABORTED: IOReturn = -536_870_165;
pub const K_KARABINER_SUCCESS_SKIP: IOReturn = 268_435_459;

#[repr(C)]
pub struct CFArrayCallBacks {
    pub version: CFIndex,
    pub retain: *const c_void,
    pub release: *const c_void,
    pub copy_description: *const c_void,
    pub equal: *const c_void,
}

#[repr(C)]
pub struct CFDictionaryKeyCallBacks {
    pub version: CFIndex,
    pub retain: *const c_void,
    pub release: *const c_void,
    pub copy_description: *const c_void,
    pub equal: *const c_void,
    pub hash: *const c_void,
}

#[repr(C)]
pub struct CFDictionaryValueCallBacks {
    pub version: CFIndex,
    pub retain: *const c_void,
    pub release: *const c_void,
    pub copy_description: *const c_void,
    pub equal: *const c_void,
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub static kCFTypeArrayCallBacks: CFArrayCallBacks;
    pub static kCFTypeDictionaryKeyCallBacks: CFDictionaryKeyCallBacks;
    pub static kCFTypeDictionaryValueCallBacks: CFDictionaryValueCallBacks;

    pub fn CFArrayCreate(
        allocator: CFAllocatorRef,
        values: *const *const c_void,
        num_values: CFIndex,
        callbacks: *const CFArrayCallBacks,
    ) -> CFArrayRef;
    pub fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(array: CFArrayRef, index: CFIndex) -> *const c_void;
    pub fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: CFIndex,
        key_callbacks: *const CFDictionaryKeyCallBacks,
        value_callbacks: *const CFDictionaryValueCallBacks,
    ) -> CFDictionaryRef;
    pub fn CFNumberCreate(
        allocator: CFAllocatorRef,
        the_type: CFNumberType,
        value_ptr: *const c_void,
    ) -> CFNumberRef;
    pub fn CFRelease(cf: CFTypeRef);
    pub fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    pub fn CFSetGetCount(set: CFSetRef) -> CFIndex;
    pub fn CFSetGetValues(set: CFSetRef, values: *mut *const c_void);
    pub fn CFStringCreateWithCString(
        allocator: CFAllocatorRef,
        c_str: *const c_char,
        encoding: CFStringEncoding,
    ) -> CFStringRef;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub fn CGEventSourceKeyState(state_id: CGEventSourceStateID, key: CGKeyCode) -> bool;
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    pub fn IOHIDDeviceCopyMatchingElements(
        device: IOHIDDeviceRef,
        matching: CFDictionaryRef,
        options: IOOptionBits,
    ) -> CFArrayRef;
    pub fn IOHIDDeviceSetValue(
        device: IOHIDDeviceRef,
        element: IOHIDElementRef,
        value: IOHIDValueRef,
    ) -> IOReturn;
    pub fn IOHIDElementGetUsage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDElementGetUsagePage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDManagerClose(manager: IOHIDManagerRef, options: IOOptionBits) -> IOReturn;
    pub fn IOHIDManagerCopyDevices(manager: IOHIDManagerRef) -> CFSetRef;
    pub fn IOHIDManagerCreate(allocator: CFAllocatorRef, options: IOOptionBits) -> IOHIDManagerRef;
    pub fn IOHIDManagerOpen(manager: IOHIDManagerRef, options: IOOptionBits) -> IOReturn;
    pub fn IOHIDManagerSetDeviceMatchingMultiple(manager: IOHIDManagerRef, multiple: CFArrayRef);
    pub fn IOHIDValueCreateWithIntegerValue(
        allocator: CFAllocatorRef,
        element: IOHIDElementRef,
        time_stamp: u64,
        value: CFIndex,
    ) -> IOHIDValueRef;
}

unsafe extern "C" {
    pub fn mach_absolute_time() -> u64;
}
