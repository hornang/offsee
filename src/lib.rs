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

    #[derive(Debug)]
    pub struct FieldTypeData {
        name: String,
        array_descriptor: String,
        format_controls: String,
    }

    #[derive(Debug)]
    pub enum ParseError {
        Generic(String),
        IOError(std::io::Error),
        FromUtf8Error(std::string::FromUtf8Error),
        DekuError(DekuError),
    }

    impl From<std::string::FromUtf8Error> for ParseError {
        fn from(err: std::string::FromUtf8Error) -> Self {
            ParseError::FromUtf8Error(err)
        }
    }

    impl From<String> for ParseError {
        fn from(err: String) -> Self {
            ParseError::Generic(err)
        }
    }

    impl From<std::io::Error> for ParseError {
        fn from(err: std::io::Error) -> Self {
            ParseError::IOError(err)
        }
    }

    impl From<DekuError> for ParseError {
        fn from(err: DekuError) -> Self {
            ParseError::DekuError(err)
        }
    }

    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ParseError::Generic(message) => write!(f, "{}", message),
                ParseError::IOError(message) => write!(f, "{}", message),
                ParseError::FromUtf8Error(io_error) => write!(f, "{}", io_error),
                ParseError::DekuError(io_error) => write!(f, "{}", io_error),
            }
        }
    }

    impl FieldType {
        pub fn data_structure(&self) -> char {
            char::from(self.data_structure)
        }
        pub fn data_type(&self) -> char {
            char::from(self.data_type)
        }
        pub fn auxiliry_controls(&self) -> String {
            String::from_utf8_lossy(&self.auxiliry_controls).into_owned()
        }
        pub fn printable_ft(&self) -> char {
            char::from(self.printable_ft)
        }
        pub fn printable_ut(&self) -> char {
            char::from(self.printable_ut)
        }
        pub fn escape_seq(&self) -> String {
            String::from_utf8_lossy(&self.escape_seq).into_owned()
        }
    }

    impl fmt::Debug for FieldType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("FieldType")
                .field("data_structure", &self.data_structure())
                .field("data_type", &self.data_type())
                .field("auxiliry_controls", &self.auxiliry_controls())
                .field("printable_ft", &self.printable_ft())
                .field("printable_ut", &self.printable_ut())
                .field("escape_seq", &self.escape_seq())
                .finish()
        }
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

    pub fn read_field_types(
        reader: &mut File,
        fields: &Vec<DirEntry>,
    ) -> Result<Vec<FieldTypeData>, ParseError> {
        use std::io::{self, Seek, SeekFrom};
        let initial_position = reader.stream_position()?;

        let length = fields.len();

        let mut field_types = Vec::new();

        for field in fields {
            let current_read_pos = reader.stream_position()?;
            let expected_position = field.position as u64 + initial_position;

            if current_read_pos != expected_position {
                return Err(ParseError::Generic(
                    "Read position is not as expected".to_string(),
                ));
            }

            let (_rest, header) = FieldType::from_reader((reader, 0))?;
            println!("{:#?}", header);

            let mut buffer = vec![0u8; field.length - 9];

            reader.read_exact(&mut buffer)?;

            const DDF_UNIT_TERMINATOR: u8 = 31;

            let parts = buffer.split(|&x| x == DDF_UNIT_TERMINATOR);

            let mut name = String::new();
            let mut array_descriptor = String::new();
            let mut format_controls = String::new();

            for (num, part) in parts.enumerate() {
                match num {
                    0 => name = String::from_utf8(part.to_vec())?,
                    1 => array_descriptor = String::from_utf8(part.to_vec())?,
                    2 => format_controls = String::from_utf8(part.to_vec())?,
                    _ => {
                        return Err(ParseError::Generic(String::from(
                            "Too many parts in field type",
                        )))
                    }
                }
            }

            field_types.push(FieldTypeData {
                name: name,
                array_descriptor: array_descriptor,
                format_controls: format_controls,
            });
        }

        Ok(field_types)
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
    ) -> Result<Vec<DirEntry>, ParseError> {

        if directory_size == 0 {
            return Err(ParseError::Generic("Directory length is zero".to_string()));
        }

        let data_length = directory_size - 1;
        let element_size = header.entry_map.size_of_length()
            + header.entry_map.size_of_position()
            + header.entry_map.size_of_tag();

        if data_length % element_size != 0 {
            return Err(ParseError::Generic(
                "Directory length does not add up to a whole number of directories".to_string(),
            ));
        }

        let num_entries = data_length / element_size;

        let mut entries: Vec<DirEntry> = Vec::new();

        for _n in 0..num_entries {
            let mut buffer = vec![0u8; header.entry_map.size_of_tag()];

            if let Err(err) = reader.read_exact(&mut buffer) {
                return Err(ParseError::Generic(format!("Read error: {err}")));
            }

            let name = String::from_utf8_lossy(&buffer).to_string();

            buffer = vec![0u8; header.entry_map.size_of_length()];

            if let Err(err) = reader.read_exact(&mut buffer) {
                return Err(ParseError::Generic(format!("Read error: {err}")));
            }

            let length_string = String::from_utf8_lossy(&buffer);
            let length = length_string.parse::<usize>();

            if let Err(err) = length {
                return Err(ParseError::Generic(format!(
                    "Failed to parse length to usize: {err}"
                )));
            }

            buffer = vec![0u8; header.entry_map.size_of_position()];

            if let Err(err) = reader.read_exact(&mut buffer) {
                return Err(ParseError::Generic(format!("Read error: {err}")));
            }

            let position_string = String::from_utf8_lossy(&buffer);
            let position = position_string.parse::<usize>();

            if let Err(err) = position {
                return Err(ParseError::Generic(format!(
                    "Failed to parse position length to usize: {err}"
                )));
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
        println!("Data descriptive record:\n{:#?}", header);

        let directories = iso8211::read_directory(&mut file, &header, directory_size).unwrap();
        println!("Directories:\n{:#?}", directories);

        let field_types = iso8211::read_field_types(&mut file, &directories);

        match field_types {
            Ok(field_types) => println!("Field types: \n{:#?}", field_types),
            Err(err) => println!("Failed to parse field types: \n{:#?}", err),
        }

        // Read 10 data records

        for n in 0..10 {
            println!("Data record {}:\n", n);
            let (header, directory_size) = iso8211::read_header(&mut file).unwrap();
            println!("{:#?}", header);

            let directories = iso8211::read_directory(&mut file, &header, directory_size).unwrap();
            println!("{:#?}", directories);

            iso8211::read_fields(&mut file, &directories);
            println!("\n");
        }
    }
}
