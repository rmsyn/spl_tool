// SPDX-License-Identifier: GPL-2.0+

#[cfg(feature = "cli")]
use std::fs;
#[cfg(feature = "cli")]
use std::io::{self, Read, Seek, Write};

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use spl_tool::{crc32, crc32_final};
use spl_tool::{Error, Result};
#[cfg(feature = "cli")]
use spl_tool::{HeaderConf, UbootSplHeader};
#[cfg(feature = "cli")]
use spl_tool::{CRC_FAILED, DEF_BACKUP, DEF_SPL_FILE, MAX_SPL_LEN, SPL_HEADER_LEN};

#[derive(clap::Parser, Debug)]
#[command(author, about, long_about = None)]
#[cfg(feature = "cli")]
struct Args {
    /// Create the SPL header
    #[arg(short = 'c', long = "create-splhdr", default_value = "false")]
    create_spl_header: bool,
    /// Fix the IMG header
    #[arg(short = 'i', long = "fix-imghdr", default_value = "false")]
    fix_img_header: bool,
    /// Provide a custom SBL_BAK_OFFSET address, default value: 0x200000
    #[arg(short = 'b', long = "sbl-bak-addr", default_value = "0")]
    sbl_bak_addr: u32,
    /// Provide a custom version, default value: 0x01010101
    #[arg(short = 'v', long = "version", default_value = "0")]
    version: u32,
    /// Provide a SPL filename
    #[arg(short = 'f', long = "file")]
    file: Option<String>,
}

fn main() -> Result<()> {
    spl_main()
}

#[cfg(not(feature = "cli"))]
fn spl_main() -> Result<()> {
    println!("The CLI application requires the `cli` feature. Please re-compile with: cargo build --features cli");
    Err(Error::RequiresCliFeature)
}

#[cfg(feature = "cli")]
fn spl_main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let file = match args.file {
        Some(f) => f,
        None => {
            log::debug!("no SPL file provided, trying {DEF_SPL_FILE}");
            DEF_SPL_FILE.to_owned()
        }
    };

    let create_spl_header = args.create_spl_header;
    let fix_img_header = args.fix_img_header;
    let version = args.version;
    let bofs = args.sbl_bak_addr;

    let conf = HeaderConf::new()
        .with_name(file.as_str())
        .with_vers(version)
        .with_bofs(bofs)
        .with_create_header(create_spl_header)
        .with_fix_image_header(fix_img_header);

    log::info!("Using SPL file: {file}");

    spl_create_header(&conf)?;
    spl_fix_image_header(&conf)?;

    Ok(())
}

#[cfg(feature = "cli")]
fn spl_create_header(conf: &HeaderConf) -> Result<()> {
    if !conf.create_header() {
        Ok(())
    } else {
        let mut header = UbootSplHeader::new();

        if conf.bofs() != 0 {
            header.set_bofs(conf.bofs());
        }
        if conf.vers() != 0 {
            header.set_vers(conf.vers());
        }

        let sofs = header.sofs();
        let bofs = header.bofs();
        let vers = header.vers();
        let name = conf.name();

        log::info!("ubsplhdr.sofs: {sofs:#x}, ubsplhdr.bofs: {bofs:#x}, ubsplhdr.vers: {vers:#x}, name: {name}");

        let mut ubootspl = [0u8; MAX_SPL_LEN];
        let sz = {
            // enter limited scope to close file after reading.
            let mut f = fs::File::open(name).map_err(|err| {
                log::error!("Error opening SPL image file {name}: {err}");
                Error::InvalidSplFile
            })?;
            f.read(&mut ubootspl).map_err(|err| {
                log::error!("Error reading from SPL image file {name}: {err}");
                Error::InvalidSplFile
            })?
        };

        if sz >= MAX_SPL_LEN {
            log::error!("File too large! Please rebuild your SPL with -Os. Maximum allowed size is {MAX_SPL_LEN} bytes.");
            Err(Error::InvalidSplLen((sz, MAX_SPL_LEN)))
        } else if sz == 0 {
            log::error!("Empty SPL file.");
            Err(Error::InvalidSplLen((sz, MAX_SPL_LEN)))
        } else {
            header.set_fsiz(sz as u32);
            let outpath = format!("{name}.normal.out");
            let mut out = fs::File::create(outpath.as_str()).map_err(|err| {
                log::error!("Error creating {outpath} file: {err}");
                Error::InvalidHeaderFile
            })?;

            let v = crc32(!0, 0x04c1_1db7, &ubootspl[..sz]);
            header.set_crcs(crc32_final(v));

            {
                // enter limited scope to remove header bytes from stack after writing
                let header_bytes: [u8; SPL_HEADER_LEN] = header.into();

                out.write_all(header_bytes.as_ref()).map_err(|err| {
                    log::error!("Error writing SPL header to {outpath} file: {err}");
                    Error::InvalidHeaderFile
                })?;
            }

            out.write_all(ubootspl[..sz].as_ref()).map_err(|err| {
                log::error!("Error writing SPL image to {outpath} file: {err}");
                Error::InvalidSplFile
            })?;

            log::info!("SPL written to {outpath} successfully.");

            Ok(())
        }
    }
}

#[cfg(feature = "cli")]
fn spl_fix_image_header(conf: &HeaderConf) -> Result<()> {
    if !conf.fix_image_header() {
        Ok(())
    } else {
        let name = conf.name();
        let mut img_bytes = [0u8; SPL_HEADER_LEN];

        let mut file = fs::File::open(name).map_err(|err| {
            log::error!("Error opening SPL image {name}: {err}");
            Error::InvalidSplFile
        })?;

        file.read_exact(&mut img_bytes).map_err(|err| {
            log::error!("Error reading header from SPL image {name}: {err}");
            Error::InvalidSplFile
        })?;

        // From `spl_tool` C implementation:
        //
        // When starting with emmc, bootrom will read 0x0 instead of partition 0. (Known issues).
        // Read GPT PMBR+Header, then write the backup address at 0x4, and write the wrong CRC
        // check value at 0x290, so that bootrom CRC check fails and jump to the backup address
        // to load the real SPL.
        let mut img_header = UbootSplHeader::try_from(img_bytes)?;

        img_header.set_bofs(if conf.bofs() != 0 {
            conf.bofs()
        } else {
            DEF_BACKUP
        });

        img_header.set_crcs(CRC_FAILED);

        file.seek(io::SeekFrom::Start(0)).map_err(|err| {
            log::error!("Error seeking to beginning of SPL image {name}: {err}");
            Error::InvalidSplFile
        })?;

        {
            // enter limited scope to remove header bytes from stack after writing
            let hdr_bytes: [u8; SPL_HEADER_LEN] = img_header.into();
            file.write_all(&hdr_bytes).map_err(|err| {
                log::error!("Error writing fixed header back to SPL image {name}: {err}");
                Error::InvalidSplFile
            })?;
        }

        log::info!("IMG {name} fixed header successfully.");

        Ok(())
    }
}
