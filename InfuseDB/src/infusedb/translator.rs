use super::file_engine::PageType;
use super::file_engine::buffer_pool::BufferPool;
use super::file_engine::utils::ReadExt;
use crate::DataType;
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Cursor, Error, ErrorKind},
    rc::Rc,
};

enum WireValue {
    Eod,
    Pointer(u32, u32),
    Value(DataType),
}
struct Translator {
    buffer_pool: Rc<RefCell<BufferPool>>,
    page_id: u32,
    index: HashMap<String, (u32, u32)>,
}

impl Translator {
    pub fn new(buffer_pool: Rc<RefCell<BufferPool>>, page_id: u32) -> Result<Self, Error> {
        let mut bp = buffer_pool.borrow_mut();
        let page = bp
            .get_page(page_id)
            .ok_or(Error::new(ErrorKind::NotFound, "Page not found"))?;

        match page.page_type {
            PageType::Index => todo!(),
            PageType::Data => todo!(),
            PageType::Overflow => todo!(),
            PageType::Free => return Err(Error::new(ErrorKind::Unsupported, "Page is free")),
        };

        Ok(Translator {
            buffer_pool,
            page_id,
            index: HashMap::new(),
        })
    }

    pub fn get(&self, key: String) -> Option<DataType> {
        None
    }

    pub fn set(&self, key: String, value: DataType) -> Result<(), Error> {
        todo!()
    }

    fn expand_pointer(&self, page: u32, offset: u32) -> Result<DataType, Error> {
        let mut bp = self.buffer_pool.borrow_mut();

        let data = &bp
            .get_page(page)
            .ok_or(Error::new(std::io::ErrorKind::Other, "Page not found"))?
            .payload;

        let mut cur = Cursor::new(data);
        cur.set_position(offset as u64);
        match self.value_parser(&mut cur)? {
            WireValue::Eod => {
                return Err(Error::new(std::io::ErrorKind::InvalidData, "Null Pointer"));
            }
            WireValue::Pointer(page, offset) => self.expand_pointer(page, offset),
            WireValue::Value(data_type) => Ok(data_type),
        }
    }

    fn kv_parser(&self, data: Vec<u8>) -> Result<HashMap<String, WireValue>, Error> {
        let mut cur = Cursor::new(&data);
        let mut pairs = HashMap::new();
        loop {
            if cur.position() == data.len() as u64 {
                break;
            }

            let key = cur.read_string()?;
            let value = self.value_parser(&mut cur)?;
            pairs.insert(key, value);
        }

        Ok(pairs)
    }

    fn value_parser(&self, cur: &mut Cursor<&Vec<u8>>) -> Result<WireValue, Error> {
        let value_type = cur.read_u8()?;
        let r = match value_type {
            0x00 => WireValue::Eod,
            0x01 => {
                let id = cur.read_u128_le()?;
                let value = DataType::Id(uuid::Uuid::from_u128(id));
                WireValue::Value(value)
            }
            0x02 => WireValue::Value(DataType::Text(cur.read_string()?)),
            0x03 => WireValue::Value(DataType::Number(cur.read_f32_le()?)),
            0x04 => WireValue::Value(DataType::Boolean(cur.read_bool()?)),
            0x05 => {
                let mut array = Vec::new();
                loop {
                    let item = self.value_parser(cur)?;
                    match item {
                        WireValue::Eod => break,
                        WireValue::Pointer(page, offset) => {
                            let ext = self.expand_pointer(page, offset)?;
                            match ext {
                                DataType::Array(ext_list) => array.extend(ext_list),
                                _ => panic!("WTF"), //TODO: handle misswriten pointers, maybe break ?
                            }
                        }
                        WireValue::Value(data_type) => {
                            array.push(data_type);
                        }
                    };
                }
                WireValue::Value(DataType::Array(array))
            }
            0x06 => {
                todo!()
            }
            0x07 => WireValue::Pointer(cur.read_u32_le()?, cur.read_u32_le()?),
            _ => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid type bit",
                ));
            }
        };
        Ok(r)
    }
}
