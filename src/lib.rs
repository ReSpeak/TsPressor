extern crate ts3plugin;
extern crate base64;
extern crate brotli2;
extern crate clipboard;

use std::io::prelude::*;
use brotli2::read::{BrotliEncoder, BrotliDecoder};
use clipboard::*;
use ts3plugin::*;

struct TsPressor;

fn ts3compress(input: &str) -> Option<String> {
    let mut compressor = BrotliEncoder::new(input.as_bytes(), 9);

    let mut buf = Vec::new();
    if let Err(_) = compressor.read_to_end(&mut buf) {
        None
    } else {
        Some(base64::encode(buf.as_slice()))
    }
}

fn ts3decompress(input: &str) -> Option<String> {
    match base64::decode(input) {
        Ok(bytes) => {
            let mut decompressor = BrotliDecoder::new(bytes.as_slice());

            let mut buf = String::new();
            match decompressor.read_to_string(&mut buf) {
                Ok(_) => Some(buf),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn sendmsg<S: AsRef<str>>(server: &Server, message: S) {
    server.send_plugin_message(message);
}

impl Plugin for TsPressor {
    fn name()        -> String { String::from("TsPressor") }
    fn version()     -> String { String::from("0.1.0") }
    fn author()      -> String { String::from("Splamy") }
    fn description() -> String { String::from("Lets you compress and send \
        messages which are larger then 1024 chars between user which have this \
        plugin") }
    fn command() -> Option<String> { Some(String::from("press")) }

    fn new(api: &TsApi) -> Result<Box<Self>, InitError> {
        api.log_or_print("Inited", "tspressor", LogLevel::Info);
        Ok(Box::new(TsPressor))
    }

    fn process_command(&mut self, api: &TsApi, server: &Server,
        _: String) -> bool {
        if let Ok(clip) = ClipboardContext::new().unwrap().get_contents() {
            if let Some(compresed_output) = ts3compress(&clip) {
                sendmsg(server, compresed_output);
                api.print_message("Ok");
            } else {
                api.log_or_print("Failed to compress", "tspressor", LogLevel::Info);
            }
        } else {
            api.log_or_print("Failed to get clipboard", "tspressor", LogLevel::Info);
        }
        true
    }

    fn plugin_message(&mut self, api: &TsApi, _: &Server,
        _: String, message: String, _: Option<&Invoker>) {
        if let Some(decomp_str) = ts3decompress(&message) {
            api.print_message(decomp_str);
        }
    }
}

create_plugin!(TsPressor);
