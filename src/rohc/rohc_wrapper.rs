#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

include!("rohc_bindings.rs");

unsafe extern "C" fn etha_rohc_defalut_random_cb(
    _comp: *const rohc_comp,
    _user_context: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    0
}

pub struct EthaRohcComp {
    comp: *mut rohc_comp,
}
unsafe impl Send for EthaRohcComp {}
impl EthaRohcComp {
    pub fn new(v2: bool) -> Result<Self, rohc_status_t> {
        let comp = EthaRohcComp {
            comp: unsafe {
                let comp = rohc_comp_new2(
                    rohc_cid_type_t::ROHC_SMALL_CID,
                    ROHC_SMALL_CID_MAX.try_into().unwrap(),
                    Some(etha_rohc_defalut_random_cb),
                    std::ptr::null_mut(),
                );
                if comp.is_null() {
                    return Err(rohc_status_t::ROHC_STATUS_ERROR);
                }
                comp
            },
        };
        comp.set_optimistic(4)?;
        if v2 {
            comp.set_v2()?;
        } else {
            comp.set_v1()?;
        }
        Ok(comp)
    }
    fn set_v1(&self) -> Result<(), rohc_status_t> {
        unsafe {
            if rohc_comp_enable_profile(self.comp, rohc_profile_t::ROHC_PROFILE_RTP)
                && rohc_comp_enable_profile(self.comp, rohc_profile_t::ROHC_PROFILE_UDP)
                && rohc_comp_enable_profile(self.comp, rohc_profile_t::ROHC_PROFILE_IP)
            {
                Ok(())
            } else {
                Err(rohc_status_t::ROHC_STATUS_ERROR)
            }
        }
    }
    fn set_v2(&self) -> Result<(), rohc_status_t> {
        unsafe {
            if rohc_comp_enable_profile(self.comp, rohc_profile_t::ROHCv2_PROFILE_IP_UDP)
                && rohc_comp_enable_profile(self.comp, rohc_profile_t::ROHCv2_PROFILE_IP_UDP_RTP)
                && rohc_comp_enable_profile(self.comp, rohc_profile_t::ROHCv2_PROFILE_IP)
            {
                Ok(())
            } else {
                Err(rohc_status_t::ROHC_STATUS_ERROR)
            }
        }
    }
    pub fn set_optimistic(&self, level: usize) -> Result<(), rohc_status_t> {
        unsafe {
            if rohc_comp_set_optimistic_approach(self.comp, level) {
                Ok(())
            } else {
                Err(rohc_status_t::ROHC_STATUS_ERROR)
            }
        }
    }
    pub fn compress(&self, input: &[u8], output: &mut [u8]) -> Result<usize, rohc_status_t> {
        unsafe {
            let pkt = rohc_buf {
                time: rohc_ts { sec: 0, nsec: 0 },
                data: input.as_ptr() as *mut u8,
                max_len: input.len(),
                offset: 0,
                len: input.len(),
            };
            let mut pkt_comp = rohc_buf {
                time: rohc_ts { sec: 0, nsec: 0 },
                data: output.as_mut_ptr(),
                max_len: output.len(),
                offset: 0,
                len: 0,
            };
            let comp_status = rohc_compress4(self.comp, pkt, &mut pkt_comp as *mut rohc_buf);
            if comp_status == rohc_status_t::ROHC_STATUS_OK {
                Ok(pkt_comp.len)
            } else {
                Err(comp_status)
            }
        }
    }
}
impl Drop for EthaRohcComp {
    fn drop(&mut self) {
        unsafe { rohc_comp_free(self.comp) };
    }
}

pub struct EthaRohcDeComp {
    decomp: *mut rohc_decomp,
}
unsafe impl Send for EthaRohcDeComp {}
impl EthaRohcDeComp {
    pub fn new(v2: bool) -> Result<Self, rohc_status_t> {
        let decomp = EthaRohcDeComp {
            decomp: unsafe {
                rohc_decomp_new2(
                    rohc_cid_type_t::ROHC_SMALL_CID,
                    ROHC_SMALL_CID_MAX.try_into().unwrap(),
                    rohc_mode_t::ROHC_U_MODE,
                )
            },
        };
        if v2 {
            decomp.set_v2()?;
        } else {
            decomp.set_v1()?;
        }
        Ok(decomp)
    }
    fn set_v1(&self) -> Result<(), rohc_status_t> {
        unsafe {
            if rohc_decomp_enable_profile(self.decomp, rohc_profile_t::ROHC_PROFILE_RTP)
                && rohc_decomp_enable_profile(self.decomp, rohc_profile_t::ROHC_PROFILE_UDP)
                && rohc_decomp_enable_profile(self.decomp, rohc_profile_t::ROHC_PROFILE_IP)
            {
                Ok(())
            } else {
                Err(rohc_status_t::ROHC_STATUS_ERROR)
            }
        }
    }
    fn set_v2(&self) -> Result<(), rohc_status_t> {
        unsafe {
            if rohc_decomp_enable_profile(self.decomp, rohc_profile_t::ROHCv2_PROFILE_IP_UDP)
                && rohc_decomp_enable_profile(
                    self.decomp,
                    rohc_profile_t::ROHCv2_PROFILE_IP_UDP_RTP,
                )
                && rohc_decomp_enable_profile(self.decomp, rohc_profile_t::ROHCv2_PROFILE_IP)
            {
                Ok(())
            } else {
                Err(rohc_status_t::ROHC_STATUS_ERROR)
            }
        }
    }
    pub fn decompress(&self, input: &[u8], output: &mut [u8]) -> Result<usize, rohc_status_t> {
        unsafe {
            let pkt = rohc_buf {
                time: rohc_ts { sec: 0, nsec: 0 },
                data: input.as_ptr() as *mut u8,
                max_len: input.len(),
                offset: 0,
                len: input.len(),
            };
            let mut pkt_decomp = rohc_buf {
                time: rohc_ts { sec: 0, nsec: 0 },
                data: output.as_mut_ptr(),
                max_len: output.len(),
                offset: 0,
                len: 0,
            };
            let comp_status = rohc_decompress3(
                self.decomp,
                pkt,
                &mut pkt_decomp as *mut rohc_buf,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if comp_status == rohc_status_t::ROHC_STATUS_OK {
                Ok(pkt_decomp.len)
            } else {
                Err(comp_status)
            }
        }
    }
}

impl Drop for EthaRohcDeComp {
    fn drop(&mut self) {
        unsafe { rohc_decomp_free(self.decomp) };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "C" {
        fn print_rohc_traces(
            priv_ctxt: *mut ::std::os::raw::c_void,
            level: rohc_trace_level_t,
            entity: rohc_trace_entity_t,
            profile: ::std::os::raw::c_int,
            format: *const ::std::os::raw::c_char,
            ...
        );
    }
    #[test]
    fn rohc_comp_decomp_basic() {
        let comp = EthaRohcComp::new(true).unwrap();
        let decomp = EthaRohcDeComp::new(true).unwrap();
        // unsafe {
        //     rohc_comp_set_traces_cb2(comp.comp, Some(print_rohc_traces), std::ptr::null_mut());

        //     rohc_decomp_set_traces_cb2(
        //         decomp.decomp,
        //         Some(print_rohc_traces),
        //         std::ptr::null_mut(),
        //     );
        // }
        let test_data: Vec<u8> = vec![
            0x45, 0x00, 0x00, 0x54, 0x00, 0x00, 0x40, 0x00, 0x40, 0x01, 0x93, 0x52, 0xc0, 0xa8,
            0x13, 0x01, 0xc0, 0xa8, 0x13, 0x05, 0x08, 0x00, 0xe9, 0xc2, 0x9b, 0x42, 0x00, 0x01,
            0x66, 0x15, 0xa6, 0x45, 0x77, 0x9b, 0x04, 0x00, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
            0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
        ];
        let mut data_comp: [u8; 100] = [0; 100];
        let mut data_decomp: [u8; 100] = [0; 100];

        for _ in 0..5 {
            let r = comp.compress(&test_data, &mut data_comp).unwrap();
            println!("comp_len = {}", r);
            println!("comp_result: {:#x?}", data_comp);
            let r = decomp
                .decompress(&data_comp[..r], &mut data_decomp)
                .unwrap();
            println!("decomp_len = {}", r);
            assert_eq!(r, test_data.len());
            println!("decomp_result: {:#x?}", data_decomp);
            assert_eq!(data_decomp[..r], test_data[..]);
        }
    }
}
