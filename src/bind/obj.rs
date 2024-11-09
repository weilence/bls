use crate::{
    cfg_obj_destroy, cfg_obj_t, isc_result_ISC_R_SUCCESS, isccfg_check_namedconf,
    BIND_CHECK_ALGORITHMS, BIND_CHECK_PLUGINS,
};

use super::{log::IscLog, mem::IscMem, parser::IscParser};

pub struct Obj<'a> {
    cfg_obj: *mut cfg_obj_t,
    parser: &'a IscParser,
}

impl<'a> Obj<'a> {
    pub fn new(parser: &'a IscParser, cfg_obj: *mut cfg_obj_t) -> Self {
        Obj { cfg_obj, parser }
    }

    pub fn check(&self, log: &IscLog, mem: &IscMem) -> bool {
        let ret = unsafe {
            isccfg_check_namedconf(
                self.cfg_obj,
                BIND_CHECK_PLUGINS | BIND_CHECK_ALGORITHMS,
                log.as_ptr(),
                mem.as_ptr(),
            )
        };

        ret == isc_result_ISC_R_SUCCESS
    }
}

impl<'a> Drop for Obj<'a> {
    fn drop(&mut self) {
        unsafe {
            cfg_obj_destroy(self.parser.as_ref(), &mut self.cfg_obj);
        }
    }
}
