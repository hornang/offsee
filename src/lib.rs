use deku::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, Read};

#[derive(Debug, DekuRead)]
#[deku(endian = "little")]
pub struct Header {
    record_length: [u8; 5],
    interchange_level: u8,
    leader_identifier: u8,
    in_line_code_identifier: u8,
    version_number: u8,
    application_indicator: u8,
    field_control_length: u16,
    base_address_of_field_data: [u8; 5],
    #[deku(bytes = "3")]
    extended_char_set_indicator: u32,
    entry_map: u32,
}

fn read_header(file_path: &str) -> Result<Header, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let (_rest, my_struct) = Header::from_reader((&mut file, 0))?;

    Ok(my_struct)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes_to_string(input: [u8; 5]) -> String {
        let result = String::from_utf8(Vec::from(input));
        match result {
            Ok(s) => s,
            Err(e) => String::new(),
        }
    }

    #[test]
    fn fiddle() {
        let header = read_header("data/US5DE10M.000").unwrap();

        println!("Raw header: {:?}", header);

        println!(
            "leader_identifier: {}",
            char::from(header.leader_identifier)
        );

        println!(
            "in_line_code_identifier: {}",
            char::from(header.in_line_code_identifier)
        );

        println!("version_number: {}", char::from(header.version_number));
        println!("record_length: {}", bytes_to_string(header.record_length));
        println!(
            "record_length: {}",
            bytes_to_string(header.base_address_of_field_data)
        );
    }
}
