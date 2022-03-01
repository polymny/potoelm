pub mod po;

use std::env::args;
use std::ffi::OsStr;
use std::fs::read_dir;

use crate::po::{to_elm, Po};

pub fn main() {
    let args = args().collect::<Vec<_>>();
    let mut pos = vec![];

    for file in read_dir(&args[1]).unwrap() {
        let file = file.unwrap();
        if file.path().extension().and_then(OsStr::to_str) != Some("po") {
            continue;
        }

        pos.push(Po::parse(
            file.path().file_stem().unwrap().to_str().unwrap(),
            file.path(),
        ));
    }

    to_elm(pos);
}
