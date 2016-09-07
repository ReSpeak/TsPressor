#[macro_use]
extern crate ts3plugin;
#[macro_use]
extern crate lazy_static;
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

fn sendback<S: AsRef<str>>(api: &TsApi, server: &Server, target: MessageReceiver, message: S) {
    if let Err(err) = match target {
        MessageReceiver::Connection(cid) => {
            match server.get_connection(cid) {
                Some(con) => con.send_message(message),
                None => Err(Error::VsCritical),
            }
        }
        MessageReceiver::Channel => {
            match server.get_own_connection_id().ok()
                .and_then(|c| server.get_connection(c)) {
                Some(own_con) => {
                    match own_con.get_channel_id().ok()
                        .and_then(|c| server.get_channel(c)) {
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

impl Plugin for TsPressor {
    fn new(api: &mut TsApi) -> Result<Box<Self>, InitError> {
        api.log_or_print("Inited", "tspressor", LogLevel::Info);
        Ok(Box::new(TsPressor))
        // Or return Err(InitError::Failure) on failure
    }

    // Implement callbacks here
    fn message(&mut self,
               api: &mut TsApi,
               server_id: ServerId,
               invoker: Invoker,
               target: MessageReceiver,
               message: String,
               _: bool)
               -> bool {
        let server = api.get_server(server_id);
        if let Some(server) = server {
            let own_id = server.get_own_connection_id();

            if Ok(invoker.get_id()) == own_id {
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

    fn shutdown(&mut self, _: &mut TsApi) {
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
               TsPressor);
