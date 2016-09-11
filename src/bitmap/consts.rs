pub const FILE_SIGNATURE: [u8; 2] = [b'B', b'M'];

/// Position of offset to pixels field in header
pub const OFFSET_TO_PIXELS_POSITION: usize = 0xA;

/// Position of size of DIB Header in header
//pub const DIB_SIZE_POSITION: usize = 0xE;

/// Position of image width in pixels in header
pub const WIDTH_POSITION: usize = 0x12;

/// Position of image height in pixels in header
pub const HEIGHT_POSITION: usize = 0x16;

/*
// It is not necessary, yet
/// Position of planes field in header
const PLANES_POSITION: usize = 0x1A;
*/

/// Position of pixel depth field in header
pub const PIXEL_DEPTH_POSITION: usize = 0x1C;

/// Position of compression type in header
pub const COMPRESSION_POSITION: usize = 0x1E;

/// Position of raw bitmap data size in bytes with padding in header
pub const RAW_BITMAP_DATA_SIZE_POSITION: usize = 0x22;

/**
 * for compression of type BI_BITFIELDS,
 * there are bit masks for rgba layers
 */

/// Red mask
pub const RED_MASK_POSITION: usize = 0x36;

/// Green mask
pub const GREEN_MASK_POSITION: usize = 0x3A;

/// Blue mask
pub const BLUE_MASK_POSITION: usize = 0x3E;

/// Alpha mask
pub const ALPHA_MASK_POSITION: usize = 0x42;

/// BI_RGB compression type
pub const BI_RGB_COMPRESSION: u32 = 0;

/// BI_BITFIELDS compression type
pub const BI_BITFIELDS_COMPRESSION: u32 = 3;