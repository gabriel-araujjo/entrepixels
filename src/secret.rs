use std::io::Read;
use std::io::Write;
use std::io::Result;

use super::bitmap::Bitmap;
use super::bitmap::Pixel;

macro_rules! mask_lsb {
    ($( $x:expr ),*) => {
        [$(match $x.trailing_zeros() {
            x if x >= 32 => 0u32,
            x @ _ => 1u32 << x,
        }),*]
    }
}

pub struct BitmapStream {
    /// masks of secret message bits
    /// max of 4 (rgba)
    masks: [u32; 4],
    /// max number of bits saved in a pixel
    bits_per_pixel: u8,
    /// bitmap
    bitmap: Bitmap,
    /// current reading/writing pixel
    cur_pixel: Option<Pixel>,
    /// bit position in pixel
    bit_pos: u8,
}

impl BitmapStream {

    pub fn from_bitmap(bitmap: Bitmap) -> BitmapStream {
        // map the lsb of a mask (I do't know how to name this)
        // E.G.:
        // 0xff0000 maps in 0x010000

        let mut masks = {
            let pixel_format = bitmap.pixel_format();
            mask_lsb![
                pixel_format.red_mask,
                pixel_format.green_mask,
                pixel_format.blue_mask,
                pixel_format.alpha_mask
            ]
        };

        // sort in reverse order to keep big endian order
        masks.sort_by(|a,b| a.cmp(b).reverse());

        // count masks different of zero
        let bits_per_pixel = masks.iter().fold(0, |sum, mask| sum + if mask != &0 {1} else {0});

        BitmapStream {
            bitmap: bitmap,
            masks: masks,
            bits_per_pixel: bits_per_pixel,
            cur_pixel: None,
            bit_pos: 0,
        }
    }

    pub fn into_bitmap(self) -> Bitmap {
        self.bitmap
    }

    fn read_bit(&mut self, pixel: &mut Pixel) -> Option<bool> {
        if self.bit_pos >= self.bits_per_pixel { return None }
        let mask = self.masks[self.bit_pos as usize];
        self.bit_pos += 1;
        match pixel.value() {
            Ok(value) => Some(value & mask != 0),
            Err(_) => None,
        }
    }

    fn write_bit(&mut self, pixel: &mut Pixel, bit: bool) -> Result<()> {
        if self.bit_pos >= self.bits_per_pixel {
            return Err(::std::io::Error::new(::std::io::ErrorKind::Other, "End of pixel"))
        }
        let mask = self.masks[self.bit_pos as usize];
        let data = if bit {
            try!(pixel.value()) | mask
        } else {
            try!(pixel.value()) & !mask
        };

        self.bit_pos += 1;
        try!(pixel.set_value(data));
        Ok(())
    }
}

impl Read for BitmapStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut byte = 0u8;
        let mut read_bytes = 0usize;
        let mut read_bits = 0u8;

        while read_bytes != buf.len() {

            let pixel = {
                match self.cur_pixel {
                    Some(ref pixel) => Some(pixel.clone()),
                    None => None,
                }
            };

            let pixel = match pixel {
                Some(mut pixel) => {
                    match self.read_bit(&mut pixel) {
                        Some(b) => {
                            debug!("read bit {}", b);
                            byte <<= 1;
                            byte |= if b {1u8} else {0u8};
                            read_bits += 1;
                            if read_bits == 8 {
                                debug!("read byte {:x}", byte);
                                buf[read_bytes] = byte;
                                read_bytes += 1;
                                read_bits = 0;
                            }
                            Some(pixel)
                        },
                        None => {
                            self.bit_pos = 0;
                            match self.bitmap.pixels_from(&self.cur_pixel).next() {
                                Some(pixel) => Some(pixel),
                                None => return Ok(read_bytes),
                            }
                        }
                    }
                },
                None => {
                    self.bit_pos = 0;
                    match self.bitmap.pixels_from(&self.cur_pixel).next() {
                        Some(pixel) => Some(pixel),
                        None => return Ok(read_bytes),
                    }
                }
            };

            self.cur_pixel = pixel;
        }

        Ok(read_bytes)
    }
}

impl Write for BitmapStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() == 0 as usize {
            return Ok(0 as usize)
        }

        let mut byte = buf[0];
        let mut write_bytes = 0usize;
        let mut write_bits = 0u8;

        while write_bytes != buf.len() {

            let pixel = {
                match self.cur_pixel {
                    Some(ref pixel) => Some(pixel.clone()),
                    None => None,
                }
            };

            let pixel = match pixel {
                Some(mut pixel) => {
                    let bit = byte & 0x80 == 0x80;
                    match self.write_bit(&mut pixel, bit) {
                        Ok(_) => {
                            byte <<= 1;
                            write_bits += 1;
                            if write_bits == 8 {
                                println!("write byte {:x}", byte);
                                write_bytes += 1;
                                write_bits = 0;
                                if write_bytes < buf.len() {
                                    byte = buf[write_bytes];
                                }
                            }
                            Some(pixel)
                        },
                        Err(_) => {
                            self.bit_pos = 0;
                            match self.bitmap.pixels_from(&self.cur_pixel).next() {
                                Some(pixel) => Some(pixel),
                                None => return Ok(write_bytes),
                            }
                        }
                    }
                },
                None => {
                    self.bit_pos = 0;
                    match self.bitmap.pixels_from(&self.cur_pixel).next() {
                        Some(pixel) => Some(pixel),
                        None => return Ok(write_bytes),
                    }
                }
            };

            self.cur_pixel = pixel;
        }

        Ok(write_bytes)
    }

    fn flush(&mut self) -> Result<()> {
        try!(self.bitmap.flush());
        Ok(())
    }
}

