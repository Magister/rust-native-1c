use std::ffi::c_void;
use std::ptr::NonNull;
use widestring::U16CStr;
use crate::memory::IMemoryManager;
use crate::types::Variant;

#[repr(u8)]
enum Interface {
    IMsgBox = 0,
    IPlatformInfo = 1,
}

#[repr(C)]
struct IMessageBoxVTable {
    confirm: unsafe extern "C" fn(&IMessageBox, *const u16, *mut Variant) -> bool,
    alert: unsafe extern "C" fn(&IMessageBox, *const u16) -> bool,
}

#[repr(C)]
struct IMessageBox {
    vtable: NonNull<IMessageBoxVTable>
}

#[repr(C)]
struct IConnectorVTable {
    _drop: unsafe extern "C" fn(&mut IConnector),
    add_error: unsafe extern "C" fn(&mut IConnector, u16, *const u16, *const u16, i64) -> bool,
    read: unsafe extern "C" fn(&mut IConnector, *const u16, *const Variant, *mut u64, *const *const u16) -> bool,
    write: unsafe extern "C" fn(&mut IConnector, *const u16, *const Variant) -> bool,
    register_profile_as: unsafe extern "C" fn(&mut IConnector, *const u16) -> bool,
    set_event_buffer_depths: unsafe extern "C" fn(&mut IConnector, u64) -> bool,
    get_event_buffer_depths: unsafe extern "C" fn(&mut IConnector) -> u64,
    external_event: unsafe extern "C" fn(&mut IConnector, *const u16, *const u16, *const u16) -> bool,
    clean_event_buffer: unsafe extern "C" fn(&mut IConnector),
    set_status_line: unsafe extern "C" fn(&mut IConnector, *const u16) -> bool,
    reset_status_line: unsafe extern "C" fn(&mut IConnector),
    get_interface: unsafe extern "C" fn(&IConnector, Interface) -> *const c_void,
}

#[repr(C)]
pub struct IConnector {
    vtable: NonNull<IConnectorVTable>,
}

impl IConnector {
    pub fn add_error(&mut self, code: u16, source: &str, descr: &str, scode: i64, manager: &mut IMemoryManager) -> bool {
        let source = manager.copy_utf16_str(source);
        let descr = manager.copy_utf16_str(descr);
        unsafe { (self.vtable.as_mut().add_error)(self, code, source, descr, scode) }
    }

    pub fn read(&mut self, prop_name: &str, value: &mut Variant, error: &mut u64, error_description: &mut String, manager: &mut IMemoryManager) -> bool {
        let prop_name = manager.copy_utf16_str(prop_name);
        let value = value as *mut Variant;
        let error = error as *mut u64;
        let mut error_description_ptr = std::ptr::null();

        let result = unsafe { (self.vtable.as_mut().read)(self, prop_name, value, error, &error_description_ptr) };
        if !error_description_ptr.is_null() {
            *error_description = unsafe { U16CStr::from_ptr_str(error_description_ptr).to_string().unwrap() };
            manager.free_memory((&mut error_description_ptr as *mut *const u16) as *mut *const c_void);
        }
        result
    }

    pub fn write(&mut self, prop_name: &str, value: &Variant, manager: &mut IMemoryManager) -> bool {
        let prop_name = manager.copy_utf16_str(prop_name);
        unsafe { (self.vtable.as_mut().write)(self, prop_name, value as *const Variant) }
    }

    pub fn register_profile_as(&mut self, profile_name: &str, manager: &mut IMemoryManager) -> bool {
        let profile_name = manager.copy_utf16_str(profile_name);
        unsafe { (self.vtable.as_mut().register_profile_as)(self, profile_name) }
    }

    pub fn set_event_buffer_depths(&mut self, depths: u64) -> bool {
        unsafe { (self.vtable.as_mut().set_event_buffer_depths)(self, depths) }
    }

    pub fn get_event_buffer_depths(&mut self) -> u64 {
        unsafe { (self.vtable.as_mut().get_event_buffer_depths)(self) }
    }

    pub fn external_event(&mut self, source: &str, message: &str, data: &str, manager: &mut IMemoryManager) -> bool {
        let source = manager.copy_utf16_str(source);
        let message = manager.copy_utf16_str(message);
        let data = manager.copy_utf16_str(data);
        unsafe { (self.vtable.as_mut().external_event)(self, source, message, data) }
    }

    pub fn clear_event_buffer(&mut self) {
        unsafe { (self.vtable.as_mut().clean_event_buffer)(self) }
    }

    pub fn set_status_line(&mut self, message: &str, manager: &mut IMemoryManager) -> bool {
        let message = manager.copy_utf16_str(message);
        unsafe { (self.vtable.as_mut().set_status_line)(self, message) }
    }

    pub fn reset_status_line(&mut self) {
        unsafe { (self.vtable.as_mut().reset_status_line)(self) }
    }

    fn get_interface(&mut self, interface: Interface) -> *const c_void {
        unsafe { (self.vtable.as_mut().get_interface)(self, interface) }
    }

    pub fn message_box_confirm(&mut self, text: &str, manager: &mut IMemoryManager) -> Result<Variant, ()> {
        let interface = self.get_interface(Interface::IMsgBox);
        if interface.is_null() {
            return Err(())
        }

        let mut ret= Variant::empty();
        let text = manager.copy_utf16_str(text);
        let interface = unsafe { &mut *(interface as *mut IMessageBox) };
        let result = unsafe { (interface.vtable.as_mut().confirm)(interface, text, &mut ret as *mut Variant) };
        if result {
            Ok(ret)
        }
        else {
            Err(())
        }
    }

    pub fn message_box_alert(&mut self, text: &str, manager: &mut IMemoryManager) -> Result<(), ()> {
        let interface = self.get_interface(Interface::IMsgBox);
        if interface.is_null() {
            return Err(())
        }

        let text = manager.copy_utf16_str(text);
        let interface = unsafe { &mut *(interface as *mut IMessageBox) };
        let result = unsafe { (interface.vtable.as_mut().alert)(interface, text) };
        if result {
            Ok(())
        }
        else {
            Err(())
        }
    }
}