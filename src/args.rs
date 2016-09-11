use std::boxed::Box;
use std::env::Args as EnvArgs;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::io::stdin;
use std::io::stdout;

use libc::isatty;
use libc::STDIN_FILENO;

use super::error::Error;

enum Reading {
    Message,
    Output,
    Input
}

pub struct Args<'a> {
    pub command: Option<String>,
    pub input: Box<Read + 'a>,
    pub output: Box<Write + 'a>,
    pub message: String,
}

impl<'a> Args<'a> {
    pub fn from_env_args(env_args: EnvArgs) -> Result<Args<'a>, Error> {
        let mut args = Args {
            command: None,
            input: Box::new(stdin()),
            output: Box::new(stdout()),
            message: String::from(""),
        };

        let mut reading: Option<Reading> = None;
        let mut input_from_stdin = true;

        for arg in env_args {
            match reading {
                Some(stuff) => {
                    match stuff {
                        Reading::Message => args.parse_message(&arg),
                        Reading::Input => {
                            try!(args.parse_input(&arg));
                            input_from_stdin = false;
                        },
                        Reading::Output => try!(args.parse_output(&arg)),
                    }
                    reading = None;
                },
                None => {
                    match arg.as_str() {
                        "--message" | "-m" => reading = Some(Reading::Message),
                        "--output" | "-o" => reading = Some(Reading::Output),
                        "--input" | "-i" => reading = Some(Reading::Input),
                        command @ "show" |
                        command @ "hide" => args.command = Some(String::from(command)),
                        _ => {},
                    }
                }
            }
        }

        if args.command.is_some() && input_from_stdin {
            try!(assert_stdin_is_piped());
        }

        Ok(args)
    }

    fn parse_message(&mut self, arg: &String) {
        self.message = String::from(arg.as_str());
    }

    fn parse_input(&mut self, arg: &String) -> Result<(), Error> {
        let file = try!(File::open(&arg));

        self.input = Box::new(file);
        Ok(())
    }

    fn parse_output(&mut self, arg: &String) -> Result<(), Error> {
        let file = try!(File::create(&arg));

        self.output = Box::new(file);
        Ok(())
    }

}

fn assert_stdin_is_piped() -> Result<(), Error> {
    unsafe {
        if isatty(STDIN_FILENO) == 0 {
            Ok(())
        } else {
            Err(Error::new("No input set, use `entrepixels --help` for more information"))
        }
    }
}