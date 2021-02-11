use std::{env, process::exit};

use ishihara::generate_plate;

fn main() {
    if let Some(text) = env::args().nth(1) {
        let image = generate_plate(&text);
        let file_name = format!("{}.png", text);
        image.save(&file_name).unwrap();
        eprintln!("Generated: {}", &file_name);
        exit(0);
    }
    exit(1);
}
