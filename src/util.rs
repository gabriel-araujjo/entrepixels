use std::u16;
use std::u32;

#[macro_export]
macro_rules! read_num_bytes {
    ($ty:ty, $size:expr, $src:expr) => ({
        assert!($size <= ::std::mem::size_of::<$ty>());
        assert!($size <= $src.len());
        let mut data: $ty = 0;
        unsafe {
            ::std::ptr::copy_nonoverlapping(
                $src.as_ptr(),
                &mut data as *mut $ty as *mut u8,
                $size);
        }
        data
    });
}


/// read an u32 from buffer at position
/// The number is in little endian format in buffer
#[inline(always)]
pub fn read_le_u32(buf: &[u8], position: usize) -> u32 {
    let n = &buf[position .. (position + 4)];
    u32::from_le(read_num_bytes!(u32, 4, n))
}

/// read an u16 from buffer at position
/// The number is in little endian format in buffer
#[inline(always)]
pub fn read_le_u16(buf: &[u8], position: usize) -> u16 {
    let n = &buf[position .. (position + 2)];
    u16::from_le(read_num_bytes!(u16, 2, n))
}

/// write an u32 number into buffer in little endian at position
#[inline(always)]
pub fn write_le_u32(buf: &mut [u8], position: usize, number: u32) {
    let n = number.to_le();
    for i in 0..4 {
        buf[position + i] = ((n >> 8 * i) & 0xff) as u8;
    }
}