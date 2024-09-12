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
            if self.size_of_length >= 48 && self.size_of_length <= 57 {
                return (self.size_of_length as u8 - 48) as usize;
            } else if self.size_of_length == 32 {
                return 0;
            }
            0
        }
        pub fn size_of_position(&self) -> usize {
            if self.size_of_position >= 48 && self.size_of_length <= 57 {
                return (self.size_of_position as u8 - 48) as usize;
            } else if self.size_of_position == 32 {
                return 0;
            }
            0
        }

        pub fn size_of_tag(&self) -> usize {
            if self.size_of_tag >= 48 && self.size_of_length <= 57 {
                return (self.size_of_tag as u8 - 48) as usize;
            } else if self.size_of_tag == 32 {
                return 0;
            }
            0
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

    #[derive(DekuRead)]
    pub struct FieldType {
        data_structure: u8,
        data_type: u8,
        auxiliry_controls: [u8; 2],
        printable_ft: u8,
        printable_ut: u8,
        escape_seq: [u8; 3],
    }

    impl fmt::Debug for RawHeader {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("RawHeader")
                .field("record_length", &self.record_length())
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
        pub tag: String,
        pub length: usize,
        pub position: usize,
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

        let mut remaining_size: usize = 0;

        if header.base_address_of_field_data() > bytes_read + 1 {
            remaining_size = header.base_address_of_field_data() - bytes_read;
        }

        Ok((header, remaining_size))
    }

    pub fn read_field_types(reader: &mut File, directories: &Vec<DirEntry>) {
        for directory in directories {
            let (_rest, header) = FieldType::from_reader((reader, 0)).unwrap();

            let mut buffer = vec![0u8; directory.length - 9];
            reader.read_exact(&mut buffer);
        }
    }

    pub fn read_fields(reader: &mut File, directories: &Vec<DirEntry>) {
        for directory in directories {
            let length = directory.length;
            let mut buffer = vec![0u8; length];
            reader.read_exact(&mut buffer);
        }
    }

    pub fn read_directory(
        reader: &mut File,
        header: &RawHeader,
        directory_size: usize,
    ) -> Result<Vec<DirEntry>, String> {
        if directory_size == 0 {
            return Err(String::from("Directory length is zero"));
        }

        let data_length = directory_size - 1;
        let element_size = header.entry_map.size_of_length()
            + header.entry_map.size_of_position()
            + header.entry_map.size_of_tag();

        if data_length % element_size != 0 {
            return Err(String::from(
                "Directory length does not add up to a whole number of directories",
            ));
        }

        let num_entries = data_length / element_size;

        let mut entries: Vec<DirEntry> = Vec::new();

        for _n in 0..num_entries {
            let mut buffer = vec![0u8; header.entry_map.size_of_tag()];

            if let Err(err) = reader.read_exact(&mut buffer) {
                return Err(format!("Read error: {err}"));
            }

            let name = String::from_utf8_lossy(&buffer).to_string();

            buffer = vec![0u8; header.entry_map.size_of_length()];

            if let Err(err) = reader.read_exact(&mut buffer) {
                return Err(format!("Read error: {err}"));
            }

            let length_string = String::from_utf8_lossy(&buffer);
            let length = length_string.parse::<usize>();

            if let Err(e) = length {
                return Err(format!("Failed to parse length to usize: {e}"));
            }

            buffer = vec![0u8; header.entry_map.size_of_position()];

            if let Err(err) = reader.read_exact(&mut buffer) {
                return Err(format!("Read error: {err}"));
            }

            let position_string = String::from_utf8_lossy(&buffer);
            let position = position_string.parse::<usize>();

            if let Err(e) = position {
                return Err(format!("Failed to parse position length to usize: {e}"));
            }

            let entry = DirEntry {
                tag: name,
                length: length.unwrap(),
                position: position.unwrap(),
            };

            entries.push(entry);
        }

        let mut buffer = vec![0u8; 1];
        if let Err(err) = reader.read_exact(&mut buffer) {
            return Err(format!("Read error: {err}"));
        }

        const FIELD_TERMINATION_BYTE: u8 = 30;

        if buffer[0] != FIELD_TERMINATION_BYTE {
            return Err(format!(
                "Directory termination character is wrong: {}. Expected: {}",
                buffer[0] as u8, FIELD_TERMINATION_BYTE as u8
            ));
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
        println!("Data descriptive record: {:?}", header);

        let directories = iso8211::read_directory(&mut file, &header, directory_size).unwrap();
        println!("{:?}", directories);

        iso8211::read_field_types(&mut file, &directories);

        // Read 10 data records
        for _n in 0..10 {
            let (header, directory_size) = iso8211::read_header(&mut file).unwrap();
            println!("Data record: {:?}", header);

            let directories = iso8211::read_directory(&mut file, &header, directory_size).unwrap();
            println!("{:?}", directories);

            iso8211::read_fields(&mut file, &directories);
        }
    }
}
