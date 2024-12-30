use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

use tdal3::Core;
fn main() {
    // Get the file path from the command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    }
    let file_path = &args[1];

    // Open the file
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            process::exit(1);
        }
    };

    // Read the file's content into a Vec<u8>
    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        eprintln!("Error reading file: {}", e);
        process::exit(1);
    }

    // Ensure the byte length is a multiple of 2
    if buffer.len() % 2 != 0 {
        eprintln!("File size is not a multiple of 2 bytes, cannot convert to Vec<u16>.");
        process::exit(1);
    }

    // Convert the bytes to a Vec<u16> in big-endian order
    let u16_vec: Vec<u16> = buffer
        .chunks(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]])) // Big-endian byte order
        .collect();

    let mut c = Core::new();
    c.load_obj(&u16_vec);
    while c.pc != u16::MAX - 1 {
        c.step();
    }
    c.dump_registers();
}
