#[cfg(test)]
mod desc_tests {
    use etha_model_generator::*;
    // const DESC_ENTRY_SIZE: usize = std::mem::size_of::<DescEntryT>();
    type DescEntryT = u32;

    mod bitfields {
        use super::*;
        #[desc_gen]
        bitfield::bitfield! {
            #[repr(C)]
            #[derive(Default, Copy, Clone)]
            pub struct FrameDesc([DescEntryT]);
            impl Debug;
            DescEntryT;
            pub addr_lo, set_addr_lo: 31, 0;
            pub size, set_size: 87, 64;
            pub fmt, set_fmt: 95, 88;
            pub id, set_id: 126, 96;
        }
    }

    type FrameDesc = bitfields::FrameDesc<[DescEntryT; 5]>;
    #[desc_gen]
    #[repr(u8)]
    enum FrameDescFmt {
        Linear = 0,
        SC = 1,
        Unknown = 2,
    }

    #[desc_gen(padding_to = 128)]
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct SCBufferEntry {
        pub next_addr: u64,
        pub addr: u64,
        pub size: u32,
        pub reserved: u32,
    }

    const RX_RESULT_DESC_SIZE: usize = 128;
    #[desc_gen(padding_to = RX_RESULT_DESC_SIZE)]
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct RxResultDesc {
        pub frame: FrameDesc,
        pub addr: u64,
    }

    #[test]
    fn gen_bitfield_desc_test() {
        let _ = FrameDesc::default();
        let fields = FrameDesc::fields().collect::<Vec<_>>();
        for f in &fields {
            println!("{:#?}", f);
        }
        assert_eq!(fields.len(), 4);
        let mut c_header = vec![];
        FrameDesc::gen_header(&HeaderType::C, &mut c_header).unwrap();
        println!("{}", String::from_utf8_lossy(&c_header));
    }

    #[test]
    fn gen_enum_desc_test() {
        let _ = FrameDescFmt::Linear;
        let _ = FrameDescFmt::SC;
        let _ = FrameDescFmt::Unknown;
        for v in FrameDescFmt::variants() {
            println!("{:#?}", v);
        }
        let mut c_header = vec![];
        FrameDescFmt::gen_header(&HeaderType::C, &mut c_header).unwrap();
        println!("{}", String::from_utf8_lossy(&c_header));
    }

    #[test]
    fn gen_struct_desc_test() {
        for f in SCBufferEntry::fields() {
            println!("{:#?}", f);
        }
        let mut c_header = vec![];
        SCBufferEntry::gen_header(&HeaderType::C, &mut c_header).unwrap();
        println!("{}", String::from_utf8_lossy(&c_header));
    }

    #[test]
    fn gen_nested_struct_desc_test() {
        for f in RxResultDesc::fields() {
            println!("{:#?}", f);
        }
        let mut c_header = vec![];
        RxResultDesc::gen_header(&HeaderType::C, &mut c_header).unwrap();
        println!("{}", String::from_utf8_lossy(&c_header));
    }
}

#[cfg(test)]
mod reg_tests {
    use etha_model_generator::*;
    define_reg! {
        EtherTypeFilter {
            fields {
                etype(RW, volatile): 15, 0;
                queue_id(RW): 23, 16;
                congestion_action(RW){blocking:0, drop:1}:30, 29;
                en(RW): 31, 31;
            }
        }
    }
    reg_map! {
        EtherRx(10) {
            et_filter(RW1C): EtherTypeFilter, 1;
        }
    }
    #[test]
    fn gen_reg_field_test() {
        let mut c_header = vec![];
        EtherTypeFilter::gen_header(&HeaderType::C, &mut c_header).unwrap();
        println!("{}", String::from_utf8_lossy(&c_header));
    }

    #[test]
    fn gen_reg_map_test() {
        let mut c_header = vec![];
        EtherRx::gen_header(&HeaderType::C, &mut c_header).unwrap();
        println!("{}", String::from_utf8_lossy(&c_header));
    }
}
