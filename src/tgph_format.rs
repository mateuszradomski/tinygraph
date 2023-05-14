use std::io::Write;

pub struct TGPH {
    magic: u32,
    version: u8,
    containers: Vec<TGPHContainer>,
}

impl Default for TGPH {
    fn default() -> Self {
        return Self {
            magic: 0x48504754,
            version: 1,
            containers: Vec::default(),
        };
    }
}

impl TGPH {
    pub fn serialize_into<W: Write>(self: &TGPH, stream: &mut W) -> Result<(), std::io::Error> {
        stream.write_all(&self.magic.to_le_bytes())?;
        stream.write_all(&self.version.to_le_bytes())?;
        stream.write_all(&(self.containers.len() as u16).to_le_bytes())?;
        for container in self.containers.iter() {
            container.serialize_into(stream)?;
        }
        Ok(())
    }

    pub fn add_container(self: &mut Self, container: TGPHContainer) {
        self.containers.push(container);
    }
}

pub enum ElementArrayType {
    U32(Vec<u32>),
    FLOAT32(Vec<f32>),
    STRING(Vec<String>),
}

impl ElementArrayType {
    fn get_index(self: &Self) -> u8 {
        match self {
            Self::U32(_) => return 1,
            Self::FLOAT32(_) => return 2,
            Self::STRING(_) => return 3,
        }
    }
}

pub struct TGPHContainer {
    name: String,
    elements: ElementArrayType,
}

impl TGPHContainer {
    fn serialize_string_into<W: Write>(stream: &mut W, string: &str) -> Result<(), std::io::Error> {
        if string.len() >= 255 {
            stream.write_all(&(0xff as u8).to_le_bytes())?;
            stream.write_all(&(string.len() as u16).to_le_bytes())?;
            stream.write_all(string.as_bytes())?;
        } else {
            stream.write_all(&(string.len() as u8).to_le_bytes())?;
            stream.write_all(string.as_bytes())?;
        }

        Ok(())
    }

    pub fn serialize_into<W: Write>(self: &Self, stream: &mut W) -> Result<(), std::io::Error> {
        TGPHContainer::serialize_string_into(stream, &self.name)?;

        stream.write_all(&self.elements.get_index().to_le_bytes())?;
        let elements_len = match &self.elements {
            ElementArrayType::U32(arr) => arr.len() as u32,
            ElementArrayType::FLOAT32(arr) => arr.len() as u32,
            ElementArrayType::STRING(arr) => arr.len() as u32,
        };

        stream.write_all(&(elements_len).to_le_bytes())?;

        if elements_len > 0 {
            match &self.elements {
                ElementArrayType::U32(arr) => {
                    for e in arr {
                        stream.write_all(&e.to_le_bytes())?;
                    }
                }
                ElementArrayType::FLOAT32(arr) => {
                    for e in arr {
                        stream.write_all(&e.to_le_bytes())?;
                    }
                }
                ElementArrayType::STRING(arr) => {
                    for e in arr {
                        TGPHContainer::serialize_string_into(stream, e)?;
                    }
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tgph_format::*;
    #[test]
    fn default_tgph() {
        let tgph = TGPH::default();
        assert_eq!(
            String::from_utf8((&tgph.magic.to_le_bytes()).to_vec()).unwrap(),
            "TGPH"
        );
    }

    #[test]
    fn write_default_tgph() {
        let tgph = TGPH::default();
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();
        assert_eq!(output_buffer, [0x54, 0x47, 0x50, 0x48, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn write_tgph_with_one_empty_container() {
        let mut tgph = TGPH::default();
        let container = TGPHContainer {
            name: "testing".into(),
            elements: ElementArrayType::U32(Vec::new()),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        expected.extend_from_slice(&(container.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(&container.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&(0 as u32).to_le_bytes()); // Element Count

        tgph.add_container(container);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }

    #[test]
    fn write_tgph_with_one_filled_container() {
        let mut tgph = TGPH::default();
        let container = TGPHContainer {
            name: "testing".into(),
            elements: ElementArrayType::U32(vec![12, 34, 56, 1 << 31]),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        expected.extend_from_slice(&(container.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(&container.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&(4 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(12 as u32).to_le_bytes());
        expected.extend_from_slice(&(34 as u32).to_le_bytes());
        expected.extend_from_slice(&(56 as u32).to_le_bytes());
        expected.extend_from_slice(&((1 << 31) as u32).to_le_bytes());

        tgph.add_container(container);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }

    #[test]
    fn write_tgph_with_one_filled_container_floats() {
        let mut tgph = TGPH::default();
        let container = TGPHContainer {
            name: "testing".into(),
            elements: ElementArrayType::FLOAT32(vec![3.14159, 1.618, 0.30000000000000004]),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        expected.extend_from_slice(&(container.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(&container.name.as_bytes());
        expected.extend_from_slice(&[2]); // Element Type
        expected.extend_from_slice(&(3 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(3.14159 as f32).to_le_bytes());
        expected.extend_from_slice(&(1.618 as f32).to_le_bytes());
        expected.extend_from_slice(&(0.30000000000000004 as f32).to_le_bytes());

        tgph.add_container(container);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }

    #[test]
    fn write_tgph_with_one_filled_container_strings() {
        let mut tgph = TGPH::default();
        let container = TGPHContainer {
            name: "testing".into(),
            elements: ElementArrayType::STRING(vec![
                "lorem".into(),
                "foxem".into(),
                "verylongstringemlatinem".into(),
            ]),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        expected.extend_from_slice(&(container.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(&container.name.as_bytes());
        expected.extend_from_slice(&[3]); // Element Type
        expected.extend_from_slice(&(3 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(5 as u8).to_le_bytes());
        expected.extend_from_slice(&("lorem".as_bytes()));
        expected.extend_from_slice(&(5 as u8).to_le_bytes());
        expected.extend_from_slice(&("foxem".as_bytes()));
        expected.extend_from_slice(&(23 as u8).to_le_bytes());
        expected.extend_from_slice(&("verylongstringemlatinem".as_bytes()));

        tgph.add_container(container);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }

    #[test]
    fn write_tgph_with_multiple_containers_different_types() {
        let mut tgph = TGPH::default();
        let container1 = TGPHContainer {
            name: "integers".into(),
            elements: ElementArrayType::U32(vec![12, 34, 56, 1 << 31]),
        };
        let container2 = TGPHContainer {
            name: "floats".into(),
            elements: ElementArrayType::FLOAT32(vec![3.14159, 1.618, 0.30000000000000004]),
        };
        let container3 = TGPHContainer {
            name: "strings".into(),
            elements: ElementArrayType::STRING(vec![
                "lorem".into(),
                "foxem".into(),
                "verylongstringemlatinem".into(),
            ]),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x03, 0x00]);
        expected.extend_from_slice(&(container1.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(&container1.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&(4 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(12 as u32).to_le_bytes());
        expected.extend_from_slice(&(34 as u32).to_le_bytes());
        expected.extend_from_slice(&(56 as u32).to_le_bytes());
        expected.extend_from_slice(&((1 << 31) as u32).to_le_bytes());
        expected.extend_from_slice(&(container2.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(&container2.name.as_bytes());
        expected.extend_from_slice(&[2]); // Element Type
        expected.extend_from_slice(&(3 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(3.14159 as f32).to_le_bytes());
        expected.extend_from_slice(&(1.618 as f32).to_le_bytes());
        expected.extend_from_slice(&(0.30000000000000004 as f32).to_le_bytes());
        expected.extend_from_slice(&(container3.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(&container3.name.as_bytes());
        expected.extend_from_slice(&[3]); // Element Type
        expected.extend_from_slice(&(3 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(5 as u8).to_le_bytes());
        expected.extend_from_slice(&("lorem".as_bytes()));
        expected.extend_from_slice(&(5 as u8).to_le_bytes());
        expected.extend_from_slice(&("foxem".as_bytes()));
        expected.extend_from_slice(&(23 as u8).to_le_bytes());
        expected.extend_from_slice(&("verylongstringemlatinem".as_bytes()));

        tgph.add_container(container1);
        tgph.add_container(container2);
        tgph.add_container(container3);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }

    #[test]
    fn long_container_name() {
        let mut tgph = TGPH::default();
        let container = TGPHContainer {
            name: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            elements: ElementArrayType::U32(vec![12, 34, 56, 1 << 31]),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        expected.extend_from_slice(&(0xff as u8).to_le_bytes());
        expected.extend_from_slice(&(container.name.len() as u16).to_le_bytes());
        expected.extend_from_slice(&container.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&(4 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(12 as u32).to_le_bytes());
        expected.extend_from_slice(&(34 as u32).to_le_bytes());
        expected.extend_from_slice(&(56 as u32).to_le_bytes());
        expected.extend_from_slice(&((1 << 31) as u32).to_le_bytes());

        tgph.add_container(container);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }

    #[test]
    fn long_container_name_and_long_string_elements() {
        let mut tgph = TGPH::default();
        let container = TGPHContainer {
            name: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            elements: ElementArrayType::STRING(vec![
                                               "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                                               "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++".into(),
            ]),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        expected.extend_from_slice(&(0xff as u8).to_le_bytes());
        expected.extend_from_slice(&(container.name.len() as u16).to_le_bytes());
        expected.extend_from_slice(&container.name.as_bytes());
        expected.extend_from_slice(&[3]); // Element Type
        expected.extend_from_slice(&(2 as u32).to_le_bytes()); // Element Count
        expected.extend_from_slice(&(0xff as u8).to_le_bytes());
        expected.extend_from_slice(&(1000 as u16).to_le_bytes());
        expected.extend_from_slice(&("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".as_bytes()));
        expected.extend_from_slice(&(0xff as u8).to_le_bytes());
        expected.extend_from_slice(&(1024 as u16).to_le_bytes());
        expected.extend_from_slice(&("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++".as_bytes()));

        tgph.add_container(container);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }
}
