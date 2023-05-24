use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Write};

pub trait BaseContainerElementType {
    fn push_element(&self, tgph: &mut TGPH, name: &str);
}

impl BaseContainerElementType for String {
    fn push_element(&self, tgph: &mut TGPH, name: &str) {
        let container = match tgph.containers.iter_mut().find(|c| c.name == name) {
            Some(v) => v,
            None => {
                let new = TGPHContainer {
                    name: name.to_string(),
                    elements: ElementArrayType::STRING(Vec::new()),
                };

                tgph.add_container(new);
                tgph.containers.last_mut().unwrap()
            }
        };

        if let ElementArrayType::STRING(elements) = &mut container.elements {
            elements.push(self.clone());
            while elements.len() > tgph.entry_limit {
                elements.remove(0);
            }
        } else {
            unreachable!();
        }
    }
}

impl BaseContainerElementType for u32 {
    fn push_element(&self, tgph: &mut TGPH, name: &str) {
        let container = match tgph.containers.iter_mut().find(|c| c.name == name) {
            Some(v) => v,
            None => {
                let new = TGPHContainer {
                    name: name.to_string(),
                    elements: ElementArrayType::U32(Vec::new()),
                };

                tgph.add_container(new);
                tgph.containers.last_mut().unwrap()
            }
        };

        if let ElementArrayType::U32(elements) = &mut container.elements {
            elements.push(*self);
            while elements.len() > tgph.entry_limit {
                elements.remove(0);
            }
        } else {
            unreachable!();
        }
    }
}

impl BaseContainerElementType for f32 {
    fn push_element(&self, tgph: &mut TGPH, name: &str) {
        let container = match tgph.containers.iter_mut().find(|c| c.name == name) {
            Some(v) => v,
            None => {
                let new = TGPHContainer {
                    name: name.to_string(),
                    elements: ElementArrayType::FLOAT32(Vec::new()),
                };

                tgph.add_container(new);
                tgph.containers.last_mut().unwrap()
            }
        };

        if let ElementArrayType::FLOAT32(elements) = &mut container.elements {
            elements.push(*self);
            while elements.len() > tgph.entry_limit {
                elements.remove(0);
            }
        } else {
            unreachable!();
        }
    }
}

pub struct TGPH {
    magic: u32,
    version: u8,
    pub containers: Vec<TGPHContainer>,

    entry_limit: usize,
}

impl Default for TGPH {
    fn default() -> Self {
        Self {
            magic: 0x48504754,
            version: 1,
            containers: Vec::default(),
            entry_limit: 1000,
        }
    }
}

impl TGPH {
    pub fn new(entry_limit: usize) -> Self {
        let mut def = Self::default();
        def.entry_limit = entry_limit;
        def
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

    pub fn deserialize_from<R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        let mut result = Self::default();
        result.magic = stream.read_u32::<LittleEndian>()?;
        result.version = stream.read_u8()?;

        let container_num = stream.read_u16::<LittleEndian>()?;

        for _ in 0..container_num {
            result
                .containers
                .push(TGPHContainer::deserialize_from(stream)?);
        }

        Ok(result)
    }

    pub fn add_container(&mut self, container: TGPHContainer) {
        self.containers.push(container);
    }

    pub fn append<T: BaseContainerElementType>(&mut self, data: T, name: &str) {
        data.push_element(self, name);
    }
}

pub enum ElementArrayType {
    U32(Vec<u32>),
    FLOAT32(Vec<f32>),
    STRING(Vec<String>),
}

impl ElementArrayType {
    fn get_index(&self) -> u8 {
        match self {
            Self::U32(_) => 1,
            Self::FLOAT32(_) => 2,
            Self::STRING(_) => 3,
        }
    }
}

pub struct TGPHContainer {
    pub name: String,
    pub elements: ElementArrayType,
}

impl TGPHContainer {
    fn serialize_string_into<W: Write>(stream: &mut W, string: &str) -> Result<(), std::io::Error> {
        if string.len() >= 255 {
            stream.write_all(&0xff_u8.to_le_bytes())?;
            stream.write_all(&(string.len() as u16).to_le_bytes())?;
            stream.write_all(string.as_bytes())?;
        } else {
            stream.write_all(&(string.len() as u8).to_le_bytes())?;
            stream.write_all(string.as_bytes())?;
        }

        Ok(())
    }

    pub fn serialize_into<W: Write>(&self, stream: &mut W) -> Result<(), std::io::Error> {
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

    fn deserialize_string_from<R: Read>(stream: &mut R) -> Result<String, std::io::Error> {
        let mut length: u16 = stream.read_u8()? as u16;

        if length == 0xff {
            length = stream.read_u16::<LittleEndian>()?;
        }

        let mut buf = vec![0u8; length as usize];
        stream.read(buf.as_mut_slice())?;

        Ok(String::from_utf8(buf).unwrap())
    }

    pub fn deserialize_from<R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        let mut result = Self {
            name: TGPHContainer::deserialize_string_from(stream)?,
            elements: ElementArrayType::U32(vec![]),
        };

        let element_type = stream.read_u8()?;
        let element_count = stream.read_u32::<LittleEndian>()?;

        let elements = match element_type {
            1 => {
                let mut elements = vec![];
                for _ in 0..element_count {
                    elements.push(stream.read_u32::<LittleEndian>()?);
                }
                ElementArrayType::U32(elements)
            }
            2 => {
                let mut elements = vec![];
                for _ in 0..element_count {
                    elements.push(stream.read_f32::<LittleEndian>()?);
                }
                ElementArrayType::FLOAT32(elements)
            }
            3 => {
                let mut elements = vec![];
                for _ in 0..element_count {
                    elements.push(TGPHContainer::deserialize_string_from(stream)?);
                }
                ElementArrayType::STRING(elements)
            }
            _ => unreachable!(), // Should error
        };

        result.elements = elements;

        Ok(result)
    }
}

#[cfg(test)]
mod serialize {
    use crate::tgph_format::*;
    use std::f32::consts::PI;

    #[test]
    fn default_tgph() {
        let tgph = TGPH::default();
        assert_eq!(
            String::from_utf8(tgph.magic.to_le_bytes().to_vec()).unwrap(),
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
        expected.extend_from_slice(container.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&0_u32.to_le_bytes()); // Element Count

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
        expected.extend_from_slice(container.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&4_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&12_u32.to_le_bytes());
        expected.extend_from_slice(&34_u32.to_le_bytes());
        expected.extend_from_slice(&56_u32.to_le_bytes());
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
            elements: ElementArrayType::FLOAT32(vec![PI, 1.618, 0.3]),
        };

        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        expected.extend_from_slice(&(container.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(container.name.as_bytes());
        expected.extend_from_slice(&[2]); // Element Type
        expected.extend_from_slice(&3_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&PI.to_le_bytes());
        expected.extend_from_slice(&1.618_f32.to_le_bytes());
        expected.extend_from_slice(&0.3_f32.to_le_bytes());

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
        expected.extend_from_slice(container.name.as_bytes());
        expected.extend_from_slice(&[3]); // Element Type
        expected.extend_from_slice(&3_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&5_u8.to_le_bytes());
        expected.extend_from_slice("lorem".as_bytes());
        expected.extend_from_slice(&5_u8.to_le_bytes());
        expected.extend_from_slice("foxem".as_bytes());
        expected.extend_from_slice(&23_u8.to_le_bytes());
        expected.extend_from_slice("verylongstringemlatinem".as_bytes());

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
            elements: ElementArrayType::FLOAT32(vec![PI, 1.618, 0.3]),
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
        expected.extend_from_slice(container1.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&4_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&12_u32.to_le_bytes());
        expected.extend_from_slice(&34_u32.to_le_bytes());
        expected.extend_from_slice(&56_u32.to_le_bytes());
        expected.extend_from_slice(&((1 << 31) as u32).to_le_bytes());
        expected.extend_from_slice(&(container2.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(container2.name.as_bytes());
        expected.extend_from_slice(&[2]); // Element Type
        expected.extend_from_slice(&3_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&PI.to_le_bytes());
        expected.extend_from_slice(&1.618_f32.to_le_bytes());
        expected.extend_from_slice(&0.3_f32.to_le_bytes());
        expected.extend_from_slice(&(container3.name.len() as u8).to_le_bytes());
        expected.extend_from_slice(container3.name.as_bytes());
        expected.extend_from_slice(&[3]); // Element Type
        expected.extend_from_slice(&3_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&5_u8.to_le_bytes());
        expected.extend_from_slice("lorem".as_bytes());
        expected.extend_from_slice(&5_u8.to_le_bytes());
        expected.extend_from_slice("foxem".as_bytes());
        expected.extend_from_slice(&23_u8.to_le_bytes());
        expected.extend_from_slice("verylongstringemlatinem".as_bytes());

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
        expected.extend_from_slice(&0xff_u8.to_le_bytes());
        expected.extend_from_slice(&(container.name.len() as u16).to_le_bytes());
        expected.extend_from_slice(container.name.as_bytes());
        expected.extend_from_slice(&[1]); // Element Type
        expected.extend_from_slice(&4_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&12_u32.to_le_bytes());
        expected.extend_from_slice(&34_u32.to_le_bytes());
        expected.extend_from_slice(&56_u32.to_le_bytes());
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
        expected.extend_from_slice(&0xff_u8.to_le_bytes());
        expected.extend_from_slice(&(container.name.len() as u16).to_le_bytes());
        expected.extend_from_slice(container.name.as_bytes());
        expected.extend_from_slice(&[3]); // Element Type
        expected.extend_from_slice(&2_u32.to_le_bytes()); // Element Count
        expected.extend_from_slice(&0xff_u8.to_le_bytes());
        expected.extend_from_slice(&1000_u16.to_le_bytes());
        expected.extend_from_slice("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".as_bytes());
        expected.extend_from_slice(&0xff_u8.to_le_bytes());
        expected.extend_from_slice(&1024_u16.to_le_bytes());
        expected.extend_from_slice("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++".as_bytes());

        tgph.add_container(container);
        let mut output_buffer = Vec::new();
        tgph.serialize_into(&mut output_buffer).unwrap();

        assert_eq!(output_buffer, expected);
    }
}

#[cfg(test)]
mod deserialize {
    use std::io::Cursor;
    use std::f32::consts::PI;

    use crate::tgph_format::*;
    #[test]
    fn deserialize_without_containers() {
        let bytes: Vec<u8> = vec![0x54, 0x47, 0x50, 0x48, 0x01, 0x00, 0x00];
        let mut cursor = Cursor::new(bytes);
        let tgph = TGPH::deserialize_from(&mut cursor).unwrap();

        assert_eq!(
            String::from_utf8(tgph.magic.to_le_bytes().to_vec()).unwrap(),
            "TGPH"
        );

        assert_eq!(tgph.version, 1);
        assert_eq!(tgph.containers.len(), 0);
    }

    #[test]
    fn deserialize_one_empty_container() {
        let mut bytes: Vec<u8> = Vec::new();
        let container_name = "test";

        bytes.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        bytes.extend_from_slice(&(container_name.len() as u8).to_le_bytes());
        bytes.extend_from_slice(container_name.as_bytes());
        bytes.extend_from_slice(&[1]); // Element Type
        bytes.extend_from_slice(&0_u32.to_le_bytes()); // Element Count

        let mut cursor = Cursor::new(bytes);
        let tgph = TGPH::deserialize_from(&mut cursor).unwrap();

        assert_eq!(
            String::from_utf8(tgph.magic.to_le_bytes().to_vec()).unwrap(),
            "TGPH"
        );

        assert_eq!(tgph.version, 1);
        assert_eq!(tgph.containers.len(), 1);

        assert_eq!(tgph.containers[0].name, container_name);
        if let ElementArrayType::U32(elements) = &tgph.containers[0].elements {
            assert_eq!(elements.len(), 0);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn deserialize_one_filled_container() {
        let mut bytes: Vec<u8> = Vec::new();
        let container_name = "bigredbarrow";
        bytes.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        bytes.extend_from_slice(&(container_name.len() as u8).to_le_bytes());
        bytes.extend_from_slice(container_name.as_bytes());
        bytes.extend_from_slice(&[1]); // Element Type
        bytes.extend_from_slice(&4_u32.to_le_bytes()); // Element Count
        bytes.extend_from_slice(&12_u32.to_le_bytes());
        bytes.extend_from_slice(&34_u32.to_le_bytes());
        bytes.extend_from_slice(&56_u32.to_le_bytes());
        bytes.extend_from_slice(&((1 << 31) as u32).to_le_bytes());

        let mut cursor = Cursor::new(bytes);
        let tgph = TGPH::deserialize_from(&mut cursor).unwrap();

        assert_eq!(
            String::from_utf8(tgph.magic.to_le_bytes().to_vec()).unwrap(),
            "TGPH"
        );

        assert_eq!(tgph.version, 1);
        assert_eq!(tgph.containers.len(), 1);

        assert_eq!(tgph.containers[0].name, container_name);
        if let ElementArrayType::U32(elements) = &tgph.containers[0].elements {
            assert_eq!(elements.len(), 4);
            assert_eq!(elements[0], 12);
            assert_eq!(elements[1], 34);
            assert_eq!(elements[2], 56);
            assert_eq!(elements[3], 1 << 31);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn deserialize_multiple_containers_different_types() {
        let mut bytes: Vec<u8> = Vec::new();
        let container_name1 = "quick";
        let container_name2 = "red fox";
        let container_name3 = "jumped over THE LAZY";
        bytes.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x03, 0x00]);
        bytes.extend_from_slice(&(container_name1.len() as u8).to_le_bytes());
        bytes.extend_from_slice(container_name1.as_bytes());
        bytes.extend_from_slice(&[1]); // Element Type
        bytes.extend_from_slice(&4_u32.to_le_bytes()); // Element Count
        bytes.extend_from_slice(&12_u32.to_le_bytes());
        bytes.extend_from_slice(&34_u32.to_le_bytes());
        bytes.extend_from_slice(&56_u32.to_le_bytes());
        bytes.extend_from_slice(&((1 << 31) as u32).to_le_bytes());
        bytes.extend_from_slice(&(container_name2.len() as u8).to_le_bytes());
        bytes.extend_from_slice(container_name2.as_bytes());
        bytes.extend_from_slice(&[2]); // Element Type
        bytes.extend_from_slice(&3_u32.to_le_bytes()); // Element Count
        bytes.extend_from_slice(&PI.to_le_bytes());
        bytes.extend_from_slice(&1.618_f32.to_le_bytes());
        bytes.extend_from_slice(&0.3_f32.to_le_bytes());
        bytes.extend_from_slice(&(container_name3.len() as u8).to_le_bytes());
        bytes.extend_from_slice(container_name3.as_bytes());
        bytes.extend_from_slice(&[3]); // Element Type
        bytes.extend_from_slice(&3_u32.to_le_bytes()); // Element Count
        bytes.extend_from_slice(&5_u8.to_le_bytes());
        bytes.extend_from_slice("lorem".as_bytes());
        bytes.extend_from_slice(&5_u8.to_le_bytes());
        bytes.extend_from_slice("foxem".as_bytes());
        bytes.extend_from_slice(&23_u8.to_le_bytes());
        bytes.extend_from_slice("verylongstringemlatinem".as_bytes());

        let mut cursor = Cursor::new(bytes);
        let tgph = TGPH::deserialize_from(&mut cursor).unwrap();

        assert_eq!(
            String::from_utf8(tgph.magic.to_le_bytes().to_vec()).unwrap(),
            "TGPH"
        );

        assert_eq!(tgph.version, 1);
        assert_eq!(tgph.containers.len(), 3);

        assert_eq!(tgph.containers[0].name, container_name1);
        if let ElementArrayType::U32(elements) = &tgph.containers[0].elements {
            assert_eq!(elements.len(), 4);
            assert_eq!(elements[0], 12);
            assert_eq!(elements[1], 34);
            assert_eq!(elements[2], 56);
            assert_eq!(elements[3], 1 << 31);
        } else {
            unreachable!();
        }

        assert_eq!(tgph.containers[1].name, container_name2);
        if let ElementArrayType::FLOAT32(elements) = &tgph.containers[1].elements {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], PI);
            assert_eq!(elements[1], 1.618);
            assert_eq!(elements[2], 0.3);
        } else {
            unreachable!();
        }

        assert_eq!(tgph.containers[2].name, container_name3);
        if let ElementArrayType::STRING(elements) = &tgph.containers[2].elements {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], "lorem");
            assert_eq!(elements[1], "foxem");
            assert_eq!(elements[2], "verylongstringemlatinem");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn deserialize_long_string() {
        let mut bytes: Vec<u8> = Vec::new();
        let container_name = "88g8g8g8g01023-123-13-12-31-23";
        bytes.extend_from_slice(&[0x54, 0x47, 0x50, 0x48, 0x01, 0x01, 0x00]);
        bytes.extend_from_slice(&0xff_u8.to_le_bytes());
        bytes.extend_from_slice(&(container_name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(container_name.as_bytes());
        bytes.extend_from_slice(&[3]); // Element Type
        bytes.extend_from_slice(&2_u32.to_le_bytes()); // Element Count
        bytes.extend_from_slice(&0xff_u8.to_le_bytes());
        bytes.extend_from_slice(&1000_u16.to_le_bytes());
        bytes.extend_from_slice("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".as_bytes());
        bytes.extend_from_slice(&0xff_u8.to_le_bytes());
        bytes.extend_from_slice(&1024_u16.to_le_bytes());
        bytes.extend_from_slice("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++".as_bytes());

        let mut cursor = Cursor::new(bytes);
        let tgph = TGPH::deserialize_from(&mut cursor).unwrap();

        assert_eq!(
            String::from_utf8(tgph.magic.to_le_bytes().to_vec()).unwrap(),
            "TGPH"
        );

        assert_eq!(tgph.version, 1);
        assert_eq!(tgph.containers.len(), 1);

        assert_eq!(tgph.containers[0].name, container_name);
        if let ElementArrayType::STRING(elements) = &tgph.containers[0].elements {
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0], "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            assert_eq!(elements[1], "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++");
        } else {
            unreachable!();
        }
    }
}
