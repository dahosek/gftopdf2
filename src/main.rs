mod gf_reader;
mod file_reader;

use std::fs::File;
use crate::gf_reader::gfreader;

fn main() {
    let mut file = File::open("data/cmr10.2602gf").unwrap();

    let font = gfreader(&mut file).unwrap();

    for character in font.chars.iter() {
        println!("[{}]", character.code);
    }

}
