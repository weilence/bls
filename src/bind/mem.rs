use bind_parser::{isc__mem_create, isc__mem_destroy, isc_mem_t};

pub struct IscMem {
    isc_mem: *mut isc_mem_t,
}

impl IscMem {
    pub fn new() -> Self {
        let mut isc_mem: *mut isc_mem_t = std::ptr::null_mut();
        unsafe {
            isc__mem_create(&mut isc_mem);
        }
        IscMem { isc_mem }
    }

    pub fn as_ptr(&self) -> *mut isc_mem_t {
        self.isc_mem
    }
}

impl Drop for IscMem {
    fn drop(&mut self) {
        unsafe {
            isc__mem_destroy(&mut self.isc_mem);
        }
    }
}
