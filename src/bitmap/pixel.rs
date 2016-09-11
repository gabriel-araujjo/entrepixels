// Only support BI_RGB and BI_BITFIELDS are current supported
use std::vec::Vec;
use std::rc::Rc;
use std::cell::RefCell;
use std::result::Result;
use std::io::Cursor;
use std::clone::Clone;

use super::super::error::Error;
use super::super::io::bitbuf::BitBuf;
use super::super::io::bitbuf::BitSeekFrom;


pub type BitmapData = Rc<RefCell<BitBuf<Cursor<Vec<u8>>>>>;


pub struct PixelFormat {
    // bits in pixel
    pub depth: u8,
    // channel masks
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub alpha_mask: u32,
}

pub struct Pixel {
    /// Shared data with bitmap
    bitmap_data: BitmapData,
    /// Offset of this pixel in `shared_data`
    offset: u32,
    /// Column
    column: u32,
    /// Row
    row: u32,
    /// Processed data
    value: u32,
    /// Pixel format
    pixel_format: PixelFormat,
    /// Flag to indicate whether value already was read
    value_already_read: bool,
}

impl Pixel {


    /// Returns a new pixel
    ///
    /// `bitmap_data` is the bitmap data
    ///
    /// `offset` is the offset (in bits) where the pixel are in shared_data
    ///
    /// `pixel_format` is a reference to the bitmap pixel format
    pub fn new(bitmap_data: BitmapData, offset: u32, column: u32, row: u32, pixel_format: PixelFormat) -> Pixel {
        Pixel {
            bitmap_data: bitmap_data,
            value: 0,
            offset: offset,
            column: column,
            row: row,
            pixel_format: pixel_format,
            value_already_read: false,
        }
    }

    /// get the pixel value
    pub fn value(&mut self) -> Result<u32, Error> {

        if ! self.value_already_read {
            self.value_already_read = true;
            let mut buf = self.bitmap_data.borrow_mut();

            // seek offset (in bits)
            try!(buf.seek(BitSeekFrom::Start(self.offset as u64)));

            self.value = try!(buf.read(self.pixel_format.depth));
        }

        Ok(self.value)
    }

    /// set data of a pixel
    pub fn set_value(&mut self, value: u32) -> Result<(), Error> {
        self.value = value & !(0xffffffff << self.pixel_format.depth);
        self.value_already_read = true;

        let mut buf = self.bitmap_data.borrow_mut();
        try!(buf.seek(BitSeekFrom::Start(self.offset as u64)));
        try!(buf.write(self.value, self.pixel_format.depth));
        Ok(())
    }

    /// returns the pixel format
//    #[inline(always)]
//    pub fn pixel_format(&self) -> &PixelFormat {
//        & self.pixel_format
//    }

    #[inline(always)]
    pub fn column(&self) -> u32 {
        self.column
    }

    #[inline(always)]
    pub fn row(&self) -> u32 {
        self.row
    }

    // TODO: channels getting and setting implementations
//    pub fn red(&self) -> f64 {
//        unimplemented!()
//    }
//
//    pub fn green(&self) -> f64 {
//        unimplemented!()
//    }
//
//    pub fn blue(&self) -> f64 {
//        unimplemented!()
//    }
//
//    pub fn alpha(&self) -> f64 {
//        unimplemented!()
//    }
//
//    pub fn set_red(&self) -> f64 {
//        unimplemented!()
//    }
//
//    pub fn set_green(&self) -> f64 {
//        unimplemented!()
//    }
//
//    pub fn set_blue(&self) -> f64 {
//        unimplemented!()
//    }
//
//    pub fn set_alpha(&self) -> f64 {
//        unimplemented!()
//    }
}

impl Clone for Pixel {
    fn clone(&self) -> Self {
        Pixel {
            bitmap_data: self.bitmap_data.clone(),
            value: self.value,
            offset: self.offset,
            row: self.row,
            column: self.column,
            pixel_format: self.pixel_format.clone(),
            value_already_read: self.value_already_read,
        }
    }
}

impl Clone for PixelFormat {
    fn clone(&self) -> Self {
        PixelFormat {
            depth: self.depth,
            red_mask: self.red_mask,
            green_mask: self.green_mask,
            blue_mask: self.blue_mask,
            alpha_mask: self.alpha_mask,
        }
    }
}