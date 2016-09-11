#[macro_use] extern crate log;
extern crate libc;

#[macro_use]
mod util;
mod error;
mod io;
mod bitmap;
mod secret;
mod args;

use std::env::args as env_args;
use std::io::Read;
use std::io::Write;
use std::process::exit;
use std::vec::Vec;

use bitmap::Bitmap;
use secret::BitmapStream;
use args::Args;
use error::Error;
use util::read_le_u32;
use util::write_le_u32;

fn usage() {
    print!(
r#"
Steganography tool
Options:
  -m, --message <message>    - specifies the message to be hidden into image.
                                 Mandatory in hide command
  -o, --output <destiny>     - sets the destiny output file.
                                 Default: stdout
  -i, --input <input_file>   - sets the input image.
                                 Default: stdin
Commands:
  show                       - shows a message hidden in image
  hide                       - hide a message into an image

Usage:
  entrepixels show [-i <input>] [-o <output>]
  entrepixels hide -m <message> [-i <input>] [-o <output>]
"#
    )
}

fn exec_command<'a>(command: String, args: &mut Args) -> Result<(), Error> {

    match command.as_str() {
        "show" => {
            let data = try!(read_data(args));
            let bitmap = try!(Bitmap::try_from(data));
            let mut buf = BitmapStream::from_bitmap(bitmap);
            let message = try!(read_message(&mut buf));

            try!(writeln!(* args.output, "{}", message));
        },
        "hide" => {
            if args.message.is_empty() {
                return Err(Error::new("Empty message"))
            }

            let data = try!(read_data(args));
            let bitmap = try!(Bitmap::try_from(data));
            let mut buf = BitmapStream::from_bitmap(bitmap);

            try!(write_message(&mut buf, &args.message));

            let bitmap = buf.into_bitmap();

            match Bitmap::try_unwrap_data(bitmap) {
                Ok(mut data) => {
                    try!((* args.output).write(&mut data[..]));
                },
                Err(_) => {
                    return Err(Error::new("Can't write output"))
                }
            }
        },
        _ => {
            return Err(Error::new("Invalid command, type `entrelinhas --help` for help"))
        }
    };

    Ok(())
}

fn read_data(args: &mut Args) -> Result<Vec<u8>, Error> {
    let mut data = Vec::new();

    try!((* args.input).read_to_end(&mut data));

    Ok(data)
}

fn write_message(bit_buf: &mut BitmapStream, message: &String) -> Result<(), Error> {
    // buffer with 4 bytes to store message size plus the string length in bytes
    let mut message_len = [0u8; 4];
    {
        write_le_u32(&mut message_len[..], 0, message.len() as u32);
    }
    try!(bit_buf.write(&message_len[..]));
    try!(write!(bit_buf, "{}", message));
    Ok(())
}

fn read_message(bit_buf: &mut BitmapStream) -> Result<String, Error> {
    let mut message_size = [0u8;4];
    try!(bit_buf.read(&mut message_size[..]));
    let message_size = read_le_u32(& message_size, 0);
    let mut data_message = vec![0; message_size as usize];
    try!(bit_buf.read(&mut data_message[..]));
    match String::from_utf8(data_message) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::new("Can't create utf-8 string")),
    }
}

fn main() {
    let mut args = match Args::from_env_args(env_args()) {
        Ok(args) => args,
        Err(err) => {
            match writeln!(&mut std::io::stderr(), "Error: {}", err) {
                Ok(_) => {},
                Err(_) => panic!("WTF!"),
            };
            exit(1);
        }
    };

    let command = match args.command {
        Some(ref s) => s.clone(),
        None => {
            usage();
            exit(1);
        }
    };

    match exec_command(command, &mut args) {
        Err(err) => {
            match writeln!(&mut std::io::stderr(), "{}", err) {
                Ok(_) => {},
                Err(_) => panic!("WTF!"),
            };
            exit(1);
        },
        _ => {},
    };
}