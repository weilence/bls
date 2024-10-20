use std::{
    ffi::CString,
    fmt::Display,
    os::raw::{c_char, c_int, c_void},
    ptr::null,
};

use bind_parser::{
    cfg_log_init, dns_log_init, dns_log_setcontext, fmemopen, fseek, ftell, isc_log,
    isc_log_create, isc_log_createchannel, isc_log_destroy, isc_log_registercategories,
    isc_log_setcontext, isc_log_t, isc_log_usechannel, isc_logcategory_t, isc_logconfig_t,
    isc_logdestination_t, isc_result_ISC_R_SUCCESS, memset,
    ns_log_init, ISC_LOG_DYNAMIC, ISC_LOG_ROLLNEVER, ISC_LOG_TOFILEDESC,
};

use super::mem::IscMem;

pub struct IscLog {
    isc_log: *mut isc_log_t,
    buffer: *mut c_void,
    destination: bind_parser::isc_logdestination,
}

impl IscLog {
    pub fn new(mem: &IscMem) -> Self {
        let mut destination: isc_logdestination_t = unsafe { std::mem::zeroed() };
        let mut logconfig: *mut isc_logconfig_t = std::ptr::null_mut();
        let mut log: *mut isc_log_t = std::ptr::null_mut();

        let categories = [
            isc_logcategory_t {
                name: "\0".as_ptr() as *const c_char,
                id: 0,
            },
            isc_logcategory_t {
                name: "unmatched\0".as_ptr() as *const c_char,
                id: 0,
            },
            isc_logcategory_t {
                name: std::ptr::null(),
                id: 0,
            },
        ]
        .as_mut_ptr();

        let buffer_size = 1024;
        let buffer = Box::into_raw(Box::new([0u8; 1024])) as *mut c_void;
        let mode = CString::new("w+").expect("CString::new failed");

        let ret: u32 = unsafe {
            isc_log_create(mem.as_ptr(), &mut log, &mut logconfig);
            isc_log_registercategories(log, categories);
            isc_log_setcontext(log);
            dns_log_init(log);
            dns_log_setcontext(log);
            cfg_log_init(log);
            ns_log_init(log);

            // isc_destination.file.stream = stderr;
            destination.file.stream = fmemopen(buffer, buffer_size, mode.as_ptr());
            destination.file.name = std::ptr::null();
            destination.file.versions = ISC_LOG_ROLLNEVER;
            destination.file.maximum_size = 0;
            isc_log_createchannel(
                logconfig,
                "stderr\0".as_ptr() as *const c_char,
                ISC_LOG_TOFILEDESC,
                ISC_LOG_DYNAMIC as c_int,
                &destination,
                0,
            );

            isc_log_usechannel(
                logconfig,
                "stderr\0".as_ptr() as *const c_char,
                null(),
                null(),
            )
        };

        if ret != isc_result_ISC_R_SUCCESS {
            panic!("isc_log_usechannel failed");
        }

        Self {
            destination,
            isc_log: log,
            buffer,
        }
    }

    pub fn as_ptr(&self) -> *mut isc_log {
        self.isc_log
    }
}

impl Display for IscLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = unsafe { std::ffi::CStr::from_ptr(self.buffer as *const i8) };
        let str = s.to_str().expect("failed to convert CStr to str");
        write!(f, "{}", str)?;
        unsafe {
            let offset = ftell(self.destination.file.stream) as u64;
            memset(self.buffer, 0, offset);
            fseek(self.destination.file.stream, 0, 0);
        }
        Ok(())
    }
}

impl Into<*mut isc_log_t> for IscLog {
    fn into(self) -> *mut isc_log_t {
        self.isc_log
    }
}

impl Drop for IscLog {
    fn drop(&mut self) {
        unsafe {
            isc_log_destroy(&mut self.isc_log);
            let _ = Box::from_raw(self.buffer as *mut [u8; 1024]);
        }
    }
}
