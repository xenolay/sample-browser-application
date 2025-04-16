#![no_std]
#![cfg_attr(not(target_os = "linux"), no_main)]
extern crate alloc;
use alloc::string::ToString;
use net_wasabi::http::HttpClient;
use noli::prelude::*;

fn main() {
    let client = HttpClient::new();
    match client.get("example.net".to_string(), 80, "/".to_string()) {
        Ok(res) => {
            print!("response: \n {:#?}", res);
        }
        Err(e) => {
            print!("error: \n {:#?}", e);
        }
    }
}

entry_point!(main);
