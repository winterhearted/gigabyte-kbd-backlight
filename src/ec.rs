use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

const EC_DATA_PORT: u64 = 0x62;
const EC_CMD_PORT: u64 = 0x66;
const EC_FCMD: u8 = 0xF8;
const EC_FDAT: u8 = 0xF9;
const EC_FBUF: u8 = 0xFA;
const EC_FBF1: u8 = 0xFB;
const EC_FBF2: u8 = 0xFC;
const EC_IBF: u8 = 0x02;

pub struct EcPort {
    file: File,
}

impl EcPort {
    pub fn open() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/port")
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::Permission
                } else {
                    Error::Io(e)
                }
            })?;
        Ok(EcPort { file })
    }

    fn port_read(&mut self, port: u64) -> Result<u8> {
        self.file.seek(SeekFrom::Start(port))?;
        let mut buf = [0u8; 1];
        self.file.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn port_write(&mut self, port: u64, value: u8) -> Result<()> {
        self.file.seek(SeekFrom::Start(port))?;
        self.file.write_all(&[value])?;
        Ok(())
    }

    fn ec_wait_ibf(&mut self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_millis(200);
        while Instant::now() < deadline {
            if self.port_read(EC_CMD_PORT)? & EC_IBF == 0 {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(1));
        }
        Err(Error::EcTimeout)
    }

    fn ec_write(&mut self, reg: u8, val: u8) -> Result<()> {
        self.ec_wait_ibf()?;
        self.port_write(EC_CMD_PORT, 0x81)?;
        self.ec_wait_ibf()?;
        self.port_write(EC_DATA_PORT, reg)?;
        self.ec_wait_ibf()?;
        self.port_write(EC_DATA_PORT, val)?;
        Ok(())
    }

    fn ec_cmd(
        &mut self,
        fcmd: u8,
        fdat: Option<u8>,
        fbuf: Option<u8>,
        fbf1: Option<u8>,
        fbf2: Option<u8>,
    ) -> Result<()> {
        if let Some(v) = fdat {
            self.ec_write(EC_FDAT, v)?;
        }
        if let Some(v) = fbuf {
            self.ec_write(EC_FBUF, v)?;
        }
        if let Some(v) = fbf1 {
            self.ec_write(EC_FBF1, v)?;
        }
        if let Some(v) = fbf2 {
            self.ec_write(EC_FBF2, v)?;
        }
        self.ec_write(EC_FCMD, fcmd)?;
        thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) -> Result<()> {
        // BGR order: FBUF=B, FBF1=R, FBF2=G
        self.ec_cmd(0xCA, Some(0x03), Some(b), Some(r), Some(g))
    }

    pub fn set_brightness(&mut self, level: u8) -> Result<()> {
        if level > 9 {
            return Err(Error::InvalidBrightness(level));
        }
        let ec_val = if level == 0 {
            0u8
        } else {
            ((level as u16 * 255 + 4) / 9) as u8
        };
        self.ec_cmd(0xC4, Some(0x02), Some(ec_val), None, None)
    }

    pub fn turn_on(&mut self) -> Result<()> {
        self.ec_cmd(0xC4, Some(0x02), Some(0xFF), None, None)
    }

    pub fn turn_off(&mut self) -> Result<()> {
        self.ec_cmd(0xC4, Some(0x02), Some(0x00), None, None)
    }
}
