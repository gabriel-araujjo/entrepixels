use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;

use super::super::error::Error;

pub enum BitSeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub struct BitBuf<T> {
    buf: T,
    current_byte: u8,
    remaining_bits: u8,
}

impl<T: Read + Write + Seek + Sized> BitBuf<T> {

    /// Wrap a `Read + Write + Seek` trait with a `BitBuf` struct
    pub fn from(buf: T) -> BitBuf<T> {
        BitBuf {
            buf: buf,
            current_byte: 0,
            remaining_bits: 0,
        }
    }

    pub fn into_inner(self) -> T {
        self.buf
    }

    /// Read `bits` bits from buffer
    /// and return a big endian read u32
    ///
    /// # Examples
    ///
    /// ```
    /// let a = try!(buf.read(6));
    ///
    /// ```
    pub fn read(&mut self, mut bits: u8) -> Result<u32, Error> {
        debug!("reading {} bits from data", bits);
        if bits > 32 {
            return Err(Error::new("Invalid number of bits"))
        }
        let mut result: u32 = 0;
        while bits > 0 && self.remaining_bits > 0 {
            bits -= 1;
            result = result << 1 | self.read_bit() as u32;
        }

        while bits >= 8 {
            bits -= 8;
            result = result << 8 | try!(self.read_byte()) as u32;
        }

        if bits > 0 {

            self.current_byte = try!(self.read_byte());
            self.remaining_bits = 8;

            while bits > 0 {
                bits -= 1;
                result = result << 1 | self.read_bit() as u32;
            }
        }

        debug!("read value {:x}", result);

        Ok(result)
    }


    ///
    /// Write `bits` last bits of `data` and
    ///
    /// return how many bits was written
    ///
    /// # Examples
    ///
    /// ```rust
    /// let file = File::open("some-file");
    ///
    /// let mut buf = BitBuf::from(file);
    ///
    /// try!(buf.write(0x7f, 7))
    ///
    /// ```
    ///
    pub fn write(&mut self, mut data: u32, mut bits: u8) -> Result<u8, Error> {
        debug!("writing {} bits to data", bits);
        if bits > 32 {
            return Err(Error::new("Invalid number of bits"))
        }
        data <<= 32 - bits;
        let mut result: u8 = 0;
        while bits > 0 && self.remaining_bits > 0 {
            try!(self.write_bit(data & 0x80000000u32 != 0u32));
            bits -= 1;
            data <<= 1;
            result += 1;
        }

        while bits >= 8 {
            try!(self.write_byte((data >> 24) as u8));
            bits -= 8;
            data <<= 8;
            result += 8;
        }

        if bits > 0 {
            self.current_byte = try!(self.read_byte());
            self.remaining_bits = 8;

            while bits > 0 {
                try!(self.write_bit(data & 0x80000000u32 != 0u32));
                bits -= 1;
                data <<= 1;
                result += 1;
            }
        }

        Ok(result)
    }

    ///
    /// Flush write buffer to destiny
    ///
    pub fn flush(&mut self) -> Result<(), Error> {
        if self.remaining_bits > 0 {
            let b = [self.current_byte];
            try!(self.buf.seek(SeekFrom::Current(-1)));
            try!(self.buf.write(&b[..]));
        };
        try!(self.buf.flush());
        Ok(())
    }

    ///
    /// Seek to `pos` position in bits on buffer
    ///
    pub fn seek(&mut self, pos: BitSeekFrom) -> Result<u64, Error> {
        try!(self.flush());
        match pos {
            BitSeekFrom::Start(pos) => {
                debug!("seeking to {:x}", pos / 8);
                let result = try!(self.buf.seek(SeekFrom::Start(pos / 8)));
                self.current_byte = try!(self.read_byte());
                self.remaining_bits = (8 - pos % 8) as u8;
                Ok(result * 8 + pos % 8)
            },
            BitSeekFrom::Current(pos) => {
                let pos = pos - self.remaining_bits as i64;

                println!("pos = {}", pos);
                println!("remaining_bits = {}", self.remaining_bits);
                let byte_pos = pos / 8 - if pos < 0 && pos % 8 != 0 {1} else {0};
                println!("byte_pos = {}", byte_pos);
                let result = try!(self.buf.seek(SeekFrom::Current(byte_pos)));
                println!("result = {}", result);

                self.current_byte = match self.read_byte() {
                    Ok(cur_byte) => {
                        cur_byte
                    },
                    Err(err) => {
                        try!(self.buf.seek(SeekFrom::Current(-byte_pos)));
                        return Err(err)
                    }
                };
                if pos >= 0 || pos % 8 == 0 {
                    self.remaining_bits = (8 - pos % 8) as u8;
                    Ok(result * 8 + (pos % 8) as u64)
                } else {
                    self.remaining_bits = (- pos % 8) as u8;
                    Ok(((result as i64 + 1) * 8 + pos % 8) as u64)
                }
            },
            BitSeekFrom::End(pos) => {
                let byte_pos = pos / 8 - if pos % 8 != 0 {1} else {0};
                let result = try!(self.buf.seek(SeekFrom::End(byte_pos)));
                self.current_byte = match self.read_byte() {
                    Ok(cur_byte) => {
                        cur_byte
                    },
                    Err(err) => {
                        try!(self.buf.seek(SeekFrom::Current(-byte_pos)));
                        return Err(err)
                    }
                };
                if pos % 8 == 0 {
                    self.remaining_bits = (8 - pos % 8) as u8;
                    Ok(result * 8 + (pos % 8) as u64)
                } else {
                    self.remaining_bits = (- pos % 8) as u8;
                    Ok(((result as i64 + 1) * 8 + pos % 8) as u64)
                }
            }
        }
    }


    fn read_bit(&mut self) -> bool {
        self.remaining_bits -= 1;
        self.current_byte & (1u8 << self.remaining_bits) != 0u8
    }

    fn write_bit(&mut self, bit: bool) -> Result<(), Error> {
        self.remaining_bits -= 1;
        if bit {
            self.current_byte |= 1u8 << self.remaining_bits;
        } else {
            self.current_byte &= !(1u8 << self.remaining_bits);
        }

        // flush current byte into buffer
        if self.remaining_bits == 0 {
            let b = [self.current_byte];
            try!(self.buf.seek(SeekFrom::Current(-1)));
            try!(self.buf.write(&b[..]));
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut sketch_buf = [0u8];
        if try!(self.buf.read(&mut sketch_buf[..])) != 1 {
            return Err(Error::new("End of file"))
        }
        Ok(sketch_buf[0])
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        let mut sketch_buf = [byte];
        if try!(self.buf.write(&mut sketch_buf[..])) != 1 {
            return Err(Error::new("End of file"))
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::BitBuf;
    use super::BitSeekFrom;

    #[test]
    fn new() {
        let data = Cursor::new(vec![0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10]);
        BitBuf::from(data);
    }

    #[test]
    fn into_inner() {
        let data = Cursor::new(vec![0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10]);
        assert_eq!(&BitBuf::from(data).into_inner().into_inner()[..], &[0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10]);
    }

    #[test]
    fn read() {
        let data = Cursor::new(vec![0x8c, 0x80, 0x41, 0x00]);
        let mut data = BitBuf::from(data);
        assert_eq!(data.read(5).unwrap(), 0x11);
        assert_eq!(data.read(4).unwrap(), 0x09);
        assert_eq!(data.read(15).unwrap(), 0x41);

        assert_eq!(&data.into_inner().into_inner()[..], &[0x8c, 0x80, 0x41, 0x00]);
    }

    #[test]
    fn write() {
        let data = Cursor::new(vec![0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10]);
        let mut data = BitBuf::from(data);
        assert_eq!(data.write(0x1234, 16).unwrap(), 16);
        assert_eq!(data.write(0x1234, 16).unwrap(), 16);
        assert_eq!(data.write(0x1234, 16).unwrap(), 16);
        assert!(data.write(0x1234, 20).is_err());

        assert_eq!(&data.into_inner().into_inner()[..], &[0x12, 0x34, 0x12, 0x34, 0x12, 0x34, 0x01, 0x23]);

        let data = Cursor::new(vec![0x00; 4]);
        let mut data = BitBuf::from(data);
        assert_eq!(data.write(0x11, 5).unwrap(), 5);
        assert_eq!(data.write(0x09, 4).unwrap(), 4);
        assert_eq!(data.write(0x41, 15).unwrap(), 14);

        assert_eq!(&data.into_inner().into_inner()[..], &[0x8c, 0x80, 0x41, 0x00]);
    }

    #[test]
    fn flush() {
        let mut raw_data = [0x00; 4];
        let data = Cursor::new(&mut raw_data[..]);
        let mut data = BitBuf::from(data);
        assert_eq!(data.write(0x11, 5).unwrap(), 5);
        assert_eq!(data.write(0x09, 4).unwrap(), 4);
        assert_eq!(data.write(0x41, 14).unwrap(), 14);

        data.flush().unwrap();

        assert_eq!(&data.into_inner().into_inner()[..], &[0x8c, 0x80, 0x82, 0x00]);
    }

    #[test]
    fn seek() {
        let data = Cursor::new(vec![0x8C, 0x80, 0x82, 0x02]);
        let mut data = BitBuf::from(data);

        assert_eq!(data.seek(BitSeekFrom::Current(0)).unwrap(), 0);
        assert_eq!(data.seek(BitSeekFrom::Current(0)).unwrap(), 0);
        assert!(data.seek(BitSeekFrom::Current(-1)).is_err());
        assert_eq!(data.seek(BitSeekFrom::Current(0)).unwrap(), 0);
        assert!(data.seek(BitSeekFrom::Current(-5)).is_err());
        assert_eq!(data.seek(BitSeekFrom::Current(0)).unwrap(), 0);
        assert!(data.seek(BitSeekFrom::Current(32)).is_err());
        assert_eq!(data.seek(BitSeekFrom::Current(0)).unwrap(), 0);


        assert_eq!(data.seek(BitSeekFrom::Current(5)).unwrap(), 5);
        assert_eq!(data.read(8).unwrap(), 0x90);

        assert_eq!(data.seek(BitSeekFrom::Current(5)).unwrap(), 18);
        assert_eq!(data.read(8).unwrap(), 0x08);

        assert_eq!(data.seek(BitSeekFrom::Current(-2)).unwrap(), 24);
        assert_eq!(data.read(7).unwrap(), 0x01);

        assert_eq!(data.seek(BitSeekFrom::Current(-10)).unwrap(), 21);
        assert_eq!(data.read(8).unwrap(), 0x40);

        assert_eq!(data.seek(BitSeekFrom::End(-32)).unwrap(), 0);
        assert!(data.seek(BitSeekFrom::End(-33)).is_err());
        assert_eq!(data.seek(BitSeekFrom::Current(0)).unwrap(), 0);
        assert_eq!(data.seek(BitSeekFrom::End(-1)).unwrap(), 31);
    }
}