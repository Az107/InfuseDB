use super::file_engine::buffer_pool::BufferPool;
use super::file_engine::utils::ReadExt;
use crate::DataType;

use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Cursor, Error, ErrorKind},
    rc::Rc,
    result,
};

pub struct PageHandler {
    buffer_pool: Rc<RefCell<BufferPool>>,
    page_id: u32,
}

impl PageHandler {
    pub fn new(buffer_pool: Rc<RefCell<BufferPool>>, page_id: u32) -> Self {
        PageHandler {
            buffer_pool,
            page_id,
        }
    }

    pub fn search(&self, key: &str) -> Option<DataType> {
        let page = self
            .buffer_pool
            .borrow_mut()
            .get_page(self.page_id)
            .unwrap();
        todo!()
    }

    pub fn get(&self, key: &str) -> Option<DataType> {
        let mut bp = self.buffer_pool.borrow_mut();
        let page = bp.get_page(self.page_id)?.clone();
        let result = kv_parser(&mut bp, &page.payload);
        if result.is_err() {
            println!("{:?}", result.err());
            return None;
        }
        let result = result.unwrap();
        Some(result.get(key)?.clone())
    }

    pub fn get_ptr(&self, offset: u32) -> Option<DataType> {
        let mut bp = self.buffer_pool.borrow_mut();
        let page = bp.get_page(self.page_id)?;
        let mut cur = Cursor::new(&page.payload);
        value_parser(&mut cur).ok()
    }

    pub fn set(&mut self, key: &str, value: DataType) -> Result<(u32, u32), Error> {
        let mut raw: Vec<u8> = Vec::new();
        raw.push(key.len() as u8);
        raw.append(&mut key.as_bytes().to_vec());
        raw.append(&mut dataType_to_bytes(value).0); //TODO: handle overflow!
        println!("{} => {:?}", key, raw);
        let mut bp = self.buffer_pool.borrow_mut();
        let page = bp
            .get_mut_page(self.page_id)
            .ok_or(Error::new(ErrorKind::NotFound, "Page Not found"))?;
        let offset = page.payload.len();
        page.append(raw).expect("Error appending");
        Ok((self.page_id, offset as u32))
    }

    pub fn list(&self) -> Vec<String> {
        let mut list = Vec::new();
        let mut bp = self.buffer_pool.borrow_mut();
        let page = bp.get_page(self.page_id).unwrap().clone();
        let result = kv_parser(&mut bp, &page.payload);
        if result.is_err() {
            return list;
        }
        let result = result.unwrap();
        for (k, _) in result {
            list.push(k);
        }
        return list;
    }
}

pub fn kv_parser(bp: &mut BufferPool, data: &Vec<u8>) -> Result<HashMap<String, DataType>, Error> {
    let mut cur = Cursor::new(data);
    let mut pairs = HashMap::new();
    loop {
        if cur.position() == data.len() as u64 {
            break;
        }

        let key = cur.read_string()?;
        let value = value_parser(&mut cur)?;
        pairs.insert(key, value);
    }

    Ok(pairs)
}

pub fn value_parser(cur: &mut Cursor<&Vec<u8>>) -> Result<DataType, Error> {
    let value_type = cur.read_u8()?;
    let r = match value_type {
        0x00 => DataType::Void,
        0x01 => {
            let id = cur.read_u128_le()?;
            DataType::Id(uuid::Uuid::from_u128(id))
        }
        0x02 => DataType::Text(cur.read_string()?),
        0x03 => DataType::Number(cur.read_f32_le()?),
        0x04 => DataType::Boolean(cur.read_bool()?),
        0x05 => {
            let mut array = Vec::new();
            loop {
                let item = value_parser(cur)?;
                match item {
                    DataType::Void => break,
                    _ => array.push(item),
                }
            }
            DataType::Array(array)
        }
        0x06 => {
            todo!()
        }
        0x07 => DataType::Pointer(cur.read_u32_le()?, cur.read_u32_le()?),
        _ => {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid type bit",
            ));
        }
    };
    Ok(r)
}

pub fn expand_pointer(bp: &mut BufferPool, page_id: u32, offset: u32) -> Result<DataType, Error> {
    let data = {
        &bp.get_page(page_id)
            .ok_or(Error::new(std::io::ErrorKind::Other, "Page not found"))?
            .payload
    };
    let mut cur = Cursor::new(data);
    cur.set_position(offset as u64);
    let item = value_parser(&mut cur)?;
    match item {
        DataType::Void => {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Null Pointer"));
        }
        DataType::Pointer(page, offset) => expand_pointer(bp, page, offset),
        _ => Ok(item),
    }
}

pub fn dataType_to_bytes(data_type: DataType) -> (Vec<u8>, Option<DataType>) {
    let mut result = Vec::new();
    match data_type {
        DataType::Void => {
            result.push(0x00);
        }
        DataType::Id(uuid) => {
            result.push(0x01);
            result.append(&mut uuid.to_bytes_le().to_vec());
        }
        DataType::Text(text) => {
            result.push(0x02);
            result.push(text.len() as u8);
            result.append(&mut text.into_bytes());
        }
        DataType::Number(num) => {
            result.push(0x03);
            result.append(&mut num.to_le_bytes().to_vec());
        }
        DataType::Boolean(bool) => {
            result.push(0x04);
            match bool {
                true => result.push(1),
                false => result.push(0),
            };
        }
        DataType::Array(data_types) => todo!(),
        DataType::Document(hash_map) => todo!(),
        DataType::Pointer(page, offset) => {
            result.push(0x07);
            result.append(&mut page.to_le_bytes().to_vec());
            result.append(&mut offset.to_le_bytes().to_vec());
        }
    };
    (result, None)
}
