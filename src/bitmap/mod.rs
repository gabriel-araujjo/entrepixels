use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::iter::Iterator;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Cursor;
use std::vec::Vec;
use std::mem;

mod consts;
mod pixel;

pub use self::pixel::PixelFormat;
pub use self::pixel::Pixel;
use super::error::Error;
use super::io::bitbuf::BitBuf;

/// Provavelmente irei excluir estes imports
use super::util::read_le_u16;
use super::util::read_le_u32;
use super::util::read_be_u32;

type SharedData = Rc<RefCell<BitBuf<Cursor<Vec<u8>>>>>;

pub struct Bitmap {
    // origin
    data: SharedData,
    // data offset
    offset: u32,
    // image dimensions
    width: u32,
    height: u32,
    // row length in bits (with padding)
    row_length: u32,
    // pixel format
    pixel_format: PixelFormat,
}

impl Bitmap {

    pub fn try_from(data: Vec<u8>) -> Result<Bitmap, Error> {

        let mut data = Cursor::new(data);

        let mut signature = [0; 2];

        try!(data.read_exact(&mut signature));

        if signature != consts::FILE_SIGNATURE {
            return Err(Error::new("Invalid file signature"))
        }

        let header = try!(Self::read_header(&mut data));

        // Image dimensions
        let offset = read_le_u32(&header, consts::OFFSET_TO_PIXELS_POSITION);
        let width = read_le_u32(&header, consts::WIDTH_POSITION);
        let height = read_le_u32(&header, consts::HEIGHT_POSITION);

        let data_size = read_le_u32(&header, consts::RAW_BITMAP_DATA_SIZE_POSITION);

        let pixel_format = try!(Self::read_pixel_format(&header));

        try!(data.seek(SeekFrom::Start(0)));

        Ok(Bitmap {
            // data
            data: Rc::new(RefCell::new(BitBuf::from(data))),
            // data offset
            offset: offset,
            // image dimensions
            width: width,
            height: height,
            // row length in bits
            row_length: data_size / height * 8,
            // pixel format
            pixel_format: pixel_format,
        })
    }

    pub fn try_unwrap_data(mut this: Self) -> Result<Vec<u8>, Bitmap> {

        let mut shared_data: SharedData = Rc::new(RefCell::new(BitBuf::from(Cursor::new(Vec::new()))));

        mem::swap(&mut shared_data, &mut this.data);

        match Rc::try_unwrap(shared_data) {
            Ok(cell) => {
                Ok(cell.into_inner().into_inner().into_inner())
            },
            Err(data) => {
                this.data = data;
                Err(this)
            }
        }
    }

//    #[inline(always)]
//    pub fn pixels<'a>(&'a mut self) -> Pixels<'a> {
//        Pixels::new(self, &None)
//    }

    #[inline(always)]
    pub fn pixels_from<'a>(&'a mut self, pos: & Option<Pixel>) -> Pixels<'a> {
        Pixels::new(self, pos)
    }

    #[inline(always)]
    pub fn pixel_format(& self) -> &PixelFormat {
        & self.pixel_format
    }

    #[inline(always)]
    pub fn flush(&mut self) -> Result<(), Error> {
        try!(self.data.borrow_mut().flush());
        Ok(())
    }

    /// Peek 256 bytes from bitmap
    #[inline(always)]
    fn read_header<T: Read + Seek>(buf: &mut T) -> Result<[u8;  256], Error> {
        let mut header = [0u8; 256];

        try!(buf.seek(SeekFrom::Start(0)));
        if try!(buf.read(&mut header)) < 14 {
            return Err(Error::new("Invalid File"))
        }
        if header[0] != b'B' || header[1] != b'M' {
            return Err(Error::new("Invalid File"))
        }
        Ok(header)
    }

    /// read the compression type from header
    #[inline(always)]
    fn read_pixel_format(buf: &[u8]) -> Result<PixelFormat, Error> {
        // Pixel depth and compression
        let depth = read_le_u16(buf, consts::PIXEL_DEPTH_POSITION) as u8;

        let compression = match read_le_u32(buf, consts::COMPRESSION_POSITION) {
            a @ consts::BI_RGB_COMPRESSION |
            a @ consts::BI_BITFIELDS_COMPRESSION => a,
            _ => return Err(Error::new("Unsupported pixel compression type")),
        };

        let masks = if depth == 24 && compression == consts::BI_RGB_COMPRESSION {
            (0xff0000u32, 0xff00u32, 0xffu32, 0u32)
        } else if compression == consts::BI_BITFIELDS_COMPRESSION {
            (
                read_be_u32(&buf, consts::RED_MASK_POSITION),
                read_be_u32(&buf, consts::GREEN_MASK_POSITION),
                read_be_u32(&buf, consts::BLUE_MASK_POSITION),
                read_be_u32(&buf, consts::ALPHA_MASK_POSITION)
            )
        } else {
            (1u32, 0u32, 0u32, 0u32)
        };


        Ok( PixelFormat {
                depth: depth,
                red_mask: masks.0,
                green_mask: masks.1,
                blue_mask: masks.2,
                alpha_mask: masks.3,
        })
    }

}

pub struct Pixels<'a> {
    bitmap: &'a Bitmap,
    cur_row: u32,
    cur_column: u32,
}

impl<'a> Pixels<'a> {

    fn new<'b>(bitmap: &'b Bitmap, pos: & Option<Pixel>) -> Pixels<'b> {
        match pos {
            &Some(ref pixel) => {
                debug!("row = {}, column = {}", pixel.row(), pixel.column());
                let mut pixels = Pixels {
                    bitmap: bitmap,
                    cur_row: pixel.row(),
                    cur_column: pixel.column(),
                };
                pixels.iterate();
                pixels
            },
            &None => {
                debug!("row = {}, column = {}", 0, 0);
                Pixels {
                    bitmap: bitmap,
                    cur_row: 0,
                    cur_column: 0,
                }
            }
        }
    }

    fn iterate(&mut self) {
        self.cur_column = match self.cur_column {
            x if x == self.bitmap.width - 1 => {
                self.cur_row += 1;
                0
            },
            x => x + 1,
        };
    }
}

impl<'a> Iterator for Pixels<'a> {
    type Item = Pixel;
    fn next(&mut self) -> Option<Self::Item> {
        if (self.cur_row, self.cur_column) == (self.bitmap.height, self.bitmap.width) {
            None
        } else {
            let offset = self.bitmap.offset * 8 +
                            self.cur_row *  self.bitmap.row_length +
                            self.cur_column * self.bitmap.pixel_format.depth as u32;

            let pixel = Pixel::new(
                self.bitmap.data.clone(),
                offset,
                self.cur_column,
                self.cur_row,
                self.bitmap.pixel_format.clone());

            self.iterate();

            Some(pixel)
        }
    }
}