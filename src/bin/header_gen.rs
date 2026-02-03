use clap::{App, Arg};
use etha_model_generator::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
fn main() {
    let matches = App::new("etha_model_header_gen")
        .author("shady83123 <shady831213@126.com>")
        .arg(
            Arg::with_name("path")
                .index(1)
                .required(true)
                .value_name("OUT_PATH")
                .validator(|path| {
                    if Path::new(path.as_str()).is_dir() {
                        Ok(())
                    } else {
                        Err(format!("{} is not dir!", path))
                    }
                })
                .help("header files output path"),
        )
        .arg(
            Arg::with_name("language")
                .short("l")
                .long("lang")
                .value_name("LANG")
                .require_delimiter(true)
                .takes_value(true)
                .validator(
                    |raw| match raw.split_whitespace().collect::<String>().as_str() {
                        "c" => Ok(()),
                        _ => return Err(String::from("only support 'c'")),
                    },
                )
                .default_value("c")
                .help("header files language"),
        )
        .get_matches();
    let path = PathBuf::from(matches.value_of("path").unwrap());
    let languages = matches.values_of("language").unwrap_or_default();
    for l in languages {
        gen_sc_desc(&path, l).expect(&format!(
            "Gen desc for '{}' to {} failed!",
            l,
            path.display()
        ));
        gen_desc(&path, l).expect(&format!(
            "Gen desc for '{}' to {} failed!",
            l,
            path.display()
        ));
        gen_ipsec_desc(&path, l).expect(&format!(
            "Gen desc for '{}' to {} failed!",
            l,
            path.display()
        ));
        gen_ring_regs(&path, l).expect(&format!(
            "Gen regs for '{}' to {} failed!",
            l,
            path.display()
        ));
        gen_regs(&path, l).expect(&format!(
            "Gen regs for '{}' to {} failed!",
            l,
            path.display()
        ));
        gen_ipsec_regs(&path, l).expect(&format!(
            "Gen regs for '{}' to {} failed!",
            l,
            path.display()
        ));
        gen_irqs(&path, l).expect(&format!(
            "Gen irqs for '{}' to {} failed!",
            l,
            path.display()
        ));
        gen_ipsec_irqs(&path, l).expect(&format!(
            "Gen irqs for '{}' to {} failed!",
            l,
            path.display()
        ));

        #[cfg(feature = "rohc")]
        {
            gen_rohc_desc(&path, l).expect(&format!(
                "Gen desc for '{}' to {} failed!",
                l,
                path.display()
            ));
            gen_rohc_regs(&path, l).expect(&format!(
                "Gen regs for '{}' to {} failed!",
                l,
                path.display()
            ));
            gen_rohc_irqs(&path, l).expect(&format!(
                "Gen irqs for '{}' to {} failed!",
                l,
                path.display()
            ));
        }
    }
}

fn gen_desc(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::etha::desc::{buffer::*, rx::*, tx::*};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_desc.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_DESC_H__")?;
    writeln!(out_file, "#define __ETHA_DESC_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    FrameDesc::gen_header(&header_ty, &mut out_file)?;
    TxCtrlDesc::gen_header(&header_ty, &mut out_file)?;
    TxStatusDesc::gen_header(&header_ty, &mut out_file)?;
    TxReqDesc::gen_header(&header_ty, &mut out_file)?;
    TxResultDesc::gen_header(&header_ty, &mut out_file)?;
    RxResultL2Desc::gen_header(&header_ty, &mut out_file)?;
    RxResultL3Desc::gen_header(&header_ty, &mut out_file)?;
    RxResultL4Desc::gen_header(&header_ty, &mut out_file)?;
    RxStatusDesc::gen_header(&header_ty, &mut out_file)?;
    RxResultDesc::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

#[cfg(feature = "rohc")]
fn gen_rohc_desc(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::rohc::desc::{req::*, resp::*};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_rohc_desc.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_ROHC_DESC_H__")?;
    writeln!(out_file, "#define __ETHA_ROHC_DESC_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    writeln!(out_file, "#include <etha_sc_desc.h>")?;
    writeln!(out_file, "typedef SCFrameDesc RohcFrameDesc;")?;
    RohcCfgDesc::gen_header(&header_ty, &mut out_file)?;
    RohcReqDesc::gen_header(&header_ty, &mut out_file)?;
    RohcStatusDesc::gen_header(&header_ty, &mut out_file)?;
    RohcResultDesc::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

fn gen_ipsec_desc(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::etha_ipsec::desc::{req::*, resp::*};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_ipsec_desc.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_IPSEC_DESC_H__")?;
    writeln!(out_file, "#define __ETHA_IPSEC_DESC_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    writeln!(out_file, "#include <etha_sc_desc.h>")?;
    writeln!(out_file, "typedef SCFrameDesc IpsecFrameDesc;")?;
    IpsecFrameCfgDesc::gen_header(&header_ty, &mut out_file)?;
    IpsecFrameFmtDesc::gen_header(&header_ty, &mut out_file)?;
    IpsecCfgDesc::gen_header(&header_ty, &mut out_file)?;
    IpsecReqDesc::gen_header(&header_ty, &mut out_file)?;
    IpsecStatusDesc::gen_header(&header_ty, &mut out_file)?;
    IpsecResultDesc::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

fn gen_sc_desc(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::desc::*;
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_sc_desc.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_SC_DESC_H__")?;
    writeln!(out_file, "#define __ETHA_SC_DESC_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    SCFrameDesc::gen_header(&header_ty, &mut out_file)?;
    SCBufferEntry::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

fn gen_ring_regs(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::reg_if::ring::*;
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_ring_regs.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_RING_REGS_H__")?;
    writeln!(out_file, "#define __ETHA_RING_REGS_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    writeln!(out_file, "#define RING_REGS_SIZE {:#x}", RING_REGS_SIZE)?;
    RingRegs::gen_header(&header_ty, &mut out_file)?;
    RingSize::gen_header(&header_ty, &mut out_file)?;
    RingPtr::gen_header(&header_ty, &mut out_file)?;
    RingStatus::gen_header(&header_ty, &mut out_file)?;
    RingCtrl::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

fn gen_regs(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::etha::{CHS, RX_ET_FILTERS, RX_TP5_FILTERS, reg_if::TopRegs};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_regs.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_REGS_H__")?;
    writeln!(out_file, "#define __ETHA_REGS_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    writeln!(out_file, "#include <etha_ring_regs.h>")?;
    TopRegs::<CHS, RX_ET_FILTERS, RX_TP5_FILTERS>::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

fn gen_ipsec_regs(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::etha_ipsec::{IPSEC_CH_NUM, IPSEC_SESSION_NUM, reg_if::TopRegs};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_ipsec_regs.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_IPSEC_REGS_H__")?;
    writeln!(out_file, "#define __ETHA_IPSEC_REGS_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    writeln!(out_file, "#include <etha_ring_regs.h>")?;
    TopRegs::<IPSEC_CH_NUM, IPSEC_SESSION_NUM>::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

#[cfg(feature = "rohc")]
fn gen_rohc_regs(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::rohc::{ROHC_CH_NUM, reg_if::TopRegs};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_rohc_regs.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_ROHC_REGS_H__")?;
    writeln!(out_file, "#define __ETHA_ROHC_REGS_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    writeln!(out_file, "#include <etha_ring_regs.h>")?;
    TopRegs::<ROHC_CH_NUM>::gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

fn gen_irqs(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::arbiter::*;
    use etha_model::etha::{CHS, Etha};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_irqs.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_IRQS_H__")?;
    writeln!(out_file, "#define __ETHA_IRQS_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    let etha = Etha::new(
        RRArbiter::<CHS>::new(),
        smoltcp::phy::Loopback::new(smoltcp::phy::Medium::Ethernet),
    );
    etha.irqs()
        .lock()
        .unwrap()
        .gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

fn gen_ipsec_irqs(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::arbiter::*;
    use etha_model::etha_ipsec::{EthaIpsec, IPSEC_CH_NUM};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_ipsec_irqs.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_IPSEC_IRQS_H__")?;
    writeln!(out_file, "#define __ETHA_IPSEC_IRQS_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    let etha_ipsec = EthaIpsec::new(RRArbiter::<IPSEC_CH_NUM>::new());
    etha_ipsec
        .irqs()
        .lock()
        .unwrap()
        .gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}

#[cfg(feature = "rohc")]
fn gen_rohc_irqs(path: &PathBuf, lang: &str) -> std::io::Result<()> {
    use etha_model::arbiter::*;
    use etha_model::rohc::{EthaRohc, ROHC_CH_NUM};
    let header_ty = match lang {
        "c" => HeaderType::C,
        &_ => todo!(),
    };
    let out_path = path.join(lang);
    if !out_path.is_dir() {
        fs::create_dir_all(&out_path)?;
    }
    let out_file_path = out_path.join("etha_rohc_irqs.h");
    let mut out_file = fs::File::create(&out_file_path)?;
    writeln!(out_file, "// This file is auto generated!")?;
    writeln!(out_file, "#ifndef __ETHA_ROHC_IRQS_H__")?;
    writeln!(out_file, "#define __ETHA_ROHC_IRQS_H__")?;
    writeln!(out_file, "#include <stdint.h>")?;
    let etha_rohc = EthaRohc::new(RRArbiter::<ROHC_CH_NUM>::new());
    etha_rohc
        .irqs()
        .lock()
        .unwrap()
        .gen_header(&header_ty, &mut out_file)?;
    writeln!(out_file, "#endif")?;
    println!("Gen {} successfully!", out_file_path.display());
    Ok(())
}
