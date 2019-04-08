use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use nesmemmap::mapper;

fn main() {
  let mut bin_seen = false;
  let mut file_seen = false;
  let mut file_name = String::new();
  for arg in env::args() {
    if !bin_seen {
      bin_seen = true;
    } else {
      file_name = arg;
      file_seen = true;
    }
  }
  if !file_seen {
    panic!("Must load a ROM file");
  }

  let path = Path::new(&file_name);
  let mut file = match File::open(&path) {
    Err(msg) => panic!("Couldn't open file: {}", msg.description()),
    Ok(file) => file,
  };
  let mut buffer = Vec::new();
  match file.read_to_end(&mut buffer) {
    Err(msg) => panic!("Couldn't read file: {}", msg.description()),
    Ok(_) => (),
  };

  let mmc = mapper::create_mapper(&buffer);
}