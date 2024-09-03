mod iso8211 {
    use deku::prelude::*;
    use std::fmt;
    use std::io::Read;

    #[derive(DekuRead)]
    pub struct EntryMap {
        size_of_length: u8,
        size_of_position: u8,
        reserved: u8,
        size_of_tag: u8,
    }

    impl EntryMap {
        pub fn size_of_length(&self) -> usize {
            (self.size_of_length as u8 - 48) as usize
        }
        pub fn size_of_position(&self) -> usize {
            (self.size_of_position as u8 - 48) as usize
        }

        pub fn size_of_tag(&self) -> usize {
            (self.size_of_tag as u8 - 48) as usize
        }
    }

    #[derive(DekuRead)]
    pub struct RawHeader {
        pub record_length: [u8; 5],
        pub interchange_level: u8,
        pub leader_identifier: u8,
        pub in_line_code_identifier: u8,
        pub version_number: u8,
        pub application_indicator: u8,
        pub field_control_length: [u8; 2],
        pub base_address_of_field_data: [u8; 5],
        #[deku(bytes = "3")]
        pub extended_char_set_indicator: u32,
        pub entry_map: EntryMap,
    }

    impl fmt::Debug for RawHeader {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("RawHeader")
                .field("interchange_level", &self.interchange_level())
                .field("leader_identifier", &self.leader_identifier())
                .field("in_line_code_identifier", &self.in_line_code_identifier())
                .field("version_number", &self.version_number())
                .field("application_indicator", &self.application_indicator)
                .field("field_control_length", &self.field_control_length())
                .field(
                    "base_address_of_field_data",
                    &self.base_address_of_field_data(),
                )
                .field("entry_map", &self.entry_map)
                .finish()
        }
    }

    impl fmt::Debug for EntryMap {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("EntryMap")
                .field("size_of_length", &self.size_of_length())
                .field("size_of_position", &self.size_of_position())
                .field("size_of_tag", &self.size_of_tag())
                .finish()
        }
    }

    #[derive(Debug)]
    pub struct DirEntry {
        tag: String,
        length: usize,
        position: usize,
    }

    impl RawHeader {
        pub fn record_length(&self) -> usize {
            bytes_to_int(Vec::from(self.record_length))
        }

        pub fn leader_identifier(&self) -> char {
            char::from(self.leader_identifier)
        }

        pub fn in_line_code_identifier(&self) -> char {
            char::from(self.in_line_code_identifier)
        }

        pub fn interchange_level(&self) -> char {
            char::from(self.interchange_level)
        }

        pub fn version_number(&self) -> char {
            char::from(self.version_number)
        }

        pub fn field_control_length(&self) -> usize {
            bytes_to_int(Vec::from(self.field_control_length))
        }

        pub fn base_address_of_field_data(&self) -> usize {
            bytes_to_int(Vec::from(self.base_address_of_field_data))
        }
    }

    fn bytes_to_int(input: Vec<u8>) -> usize {
        let result = String::from_utf8(input);

        match result {
            Ok(s) => {
                let number = s.parse::<usize>();

                match number {
                    Ok(v) => v,
                    Err(_) => 0,
                }
            }
            Err(e) => 0,
        }
    }

    use std::fs::File;

    pub fn read_header(
        reader: &mut File,
    ) -> Result<(RawHeader, usize), Box<dyn std::error::Error>> {
        let (_rest, header) = RawHeader::from_reader((reader, 0))?;

        let bytes_read = _rest / 8;

        let remaining_size = header.base_address_of_field_data() - bytes_read - 1;

        Ok((header, remaining_size))
    }

    pub fn read_directory(
        reader: &mut File,
        header: &RawHeader,
        directory_size: usize,
    ) -> Result<Vec<DirEntry>, String> {
        let num_entries = directory_size
            / (header.entry_map.size_of_length()
                + header.entry_map.size_of_position()
                + header.entry_map.size_of_tag());

        let mut entries: Vec<DirEntry> = Vec::new();

        for n in 0..num_entries {
            let mut buffer = vec![0u8; header.entry_map.size_of_tag()];
            reader.read_exact(&mut buffer);
            let name = String::from_utf8_lossy(&buffer).to_string();

            buffer = vec![0u8; header.entry_map.size_of_length()];
            reader.read_exact(&mut buffer);
            let length = String::from_utf8_lossy(&buffer).parse::<usize>();

            match length {
                Ok(length) => (),
                Err(_) => {
                    println!("Err");
                    return Err(String::from("Err with length"));
                }
            };

            buffer = vec![0u8; header.entry_map.size_of_position()];
            reader.read_exact(&mut buffer);
            let position = String::from_utf8_lossy(&buffer).parse::<usize>();

            match position {
                Ok(position) => (),
                Err(_) => {
                    println!("Err");
                    return Err(String::from("Err with position"));
                }
            };

            let entry = DirEntry {
                tag: name,
                length: length.unwrap(),
                position: position.unwrap(),
            };

            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fiddle() {
        let mut file = std::fs::File::open("data/ENC_ROOT/US2EC03M/US2EC03M.000").unwrap();

        let (header, directory_size) = iso8211::read_header(&mut file).unwrap();
        println!("{:?}", header);

        let directories = iso8211::read_directory(&mut file, &header, directory_size).unwrap();
        println!("{:?}", directories);
    }
}
