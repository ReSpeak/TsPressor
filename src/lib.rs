#[macro_use]
extern crate ts3plugin;
#[macro_use]
extern crate lazy_static;
extern crate clipboard;
extern crate rustc_serialize;
extern crate bzip2;

use ts3plugin::*;
use clipboard::*;
use std::io::prelude::*;
use bzip2::Compression;
use bzip2::read::{BzEncoder, BzDecoder};
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};

struct MyTsPlugin;

fn ts3compress(inp: &str) -> Option<String> {
    let bytes = inp.as_bytes();
    let mut compressor = BzEncoder::new(bytes, Compression::Best);

    let mut buf: Vec<u8> = Vec::new();
    if let Err(_) = compressor.read_to_end(&mut buf) {
        return None;
    }
    Some(buf.as_slice().to_base64(STANDARD))
}

fn ts3decompress(inp: &str) -> Option<String> {
    match inp.from_base64() {
        Ok(bytes) => {
            let mut decompressor = BzDecoder::new(bytes.as_slice());

            let mut buf = String::new();
            match decompressor.read_to_string(&mut buf) {
                Ok(_) => Some(buf),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn sendback<S: AsRef<str>>(api: &TsApi, server: &Server, target: MessageReceiver, message: S) {
    if let Err(err) = match target {
        MessageReceiver::Connection(cid) => {
            match server.get_connection(cid) {
                Some(con) => con.send_message(message),
                None => Err(Error::VsCritical),
            }
        }
        MessageReceiver::Channel => {
            let own_id = server.get_own_connection_id();
            match server.get_connection(own_id) {
                Some(own_con) => {
                    match server.get_channel(own_con.get_channel_id()) {
                        Some(chan) => chan.send_message(message),
                        None => Err(Error::VsCritical),
                    }
                }
                None => Err(Error::VsCritical),
            }
        }
        MessageReceiver::Server => server.send_message(message),
    } {
        api.log_or_print(format!("Failed to send: {:?}", err),
                         "tspressor",
                         LogLevel::Info);
    }
}

impl Plugin for MyTsPlugin {
    fn new(api: &TsApi) -> Result<Box<MyTsPlugin>, InitError> {
        api.log_or_print("Inited", "tspressor", LogLevel::Info);
        Ok(Box::new(MyTsPlugin))
        // Or return Err(InitError::Failure) on failure
    }

    // Implement callbacks here
    fn message(&mut self,
               api: &TsApi,
               server_id: ServerId,
               invoker: Invoker,
               target: MessageReceiver,
               message: String,
               _: bool)
               -> bool {
        let server = api.get_server(server_id);
        if let Some(server) = server {
            let own_id = server.get_own_connection_id();

            if invoker.get_id() == own_id {
                // own message
                if message.starts_with("--pack") {
                    if let Ok(clip) = ClipboardContext::new().unwrap().get_contents() {
                        if let Some(compresed_output) = ts3compress(&clip) {
                            sendback(api, server, target, compresed_output);
                            api.print_message("Ok");
                        } else {
                            api.log_or_print("failed to compress", "tspressor", LogLevel::Info);
                        }
                    } else {
                        api.log_or_print("failed to get clip", "tspressor", LogLevel::Info);
                    }
                    return true;
                }
            } else {
                if let Some(decomp_str) = ts3decompress(&message) {
                    api.print_message(decomp_str);
                    return true;
                }
            }
        } else {
            api.log_or_print("Message from no server", "tspressor", LogLevel::Info);
        }
        false
    }

    fn shutdown(&mut self, _: &TsApi) {
        // api.log_or_print("TsPressor says goodbyte!", "tspressor", LogLevel::Info);
    }
}

create_plugin!("TsPressor",
               "0.1.0",
               "Splamy",
               "Lets you compress and send messages which are larger then 1024 chars between \
                user which have this plugin",
               ConfigureOffer::No,
               false,
               MyTsPlugin);