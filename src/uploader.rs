use std::fs::File;
use std::io::prelude::*;

/// write_file will write the given content to a file named f_name
pub fn write_file_local(f_name:&str, content: String) {
    let mut file = File::create(f_name)
        .expect(&format!("Could not create file {}", f_name));
    file
        .write_all(
            content
            .as_bytes()
        )
        .expect(&format!("Could not write to file {}", f_name));

}
