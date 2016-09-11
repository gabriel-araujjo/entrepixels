use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;

use super::super::error::Error;

pub enum BitSeekFrom {
    Start(u64),
//    End(i64),
//    Current(i64),
}

pub struct BitBuf<T> where T: Read + Write + Seek + Sized {
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
        println!("writing {} bits to data", bits);
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
//            BitSeekFrom::Current(pos) => {
//                let pos = pos - self.remaining_bits as i64;
//                let byte_pos = pos / 8 - if pos < 8 {1} else {0};
//                let result = try!(self.buf.seek(SeekFrom::Current(byte_pos)));
//                self.current_byte = try!(self.read_byte());
//                if pos > 0 {
//                    self.remaining_bits = (8 - pos % 8) as u8;
//                    Ok(result * 8 + (pos % 8) as u64)
//                } else {
//                    self.remaining_bits = (- pos % 8) as u8;
//                    Ok((result + 1) * 8 + (pos % 8) as u64)
//                }
//            },
//            BitSeekFrom::End(pos) => {
//                let result = try!(self.buf.seek(SeekFrom::End(pos / 8 - 1)));
//                self.current_byte = try!(self.read_byte());
//                self.remaining_bits = (- pos % 8) as u8;
//                Ok((result + 1) * 8 + (pos % 8) as u64)
//            }
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