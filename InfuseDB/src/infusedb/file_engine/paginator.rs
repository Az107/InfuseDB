use std::{
    fs::{File, OpenOptions},
    io::{Cursor, Error, Read, Seek, SeekFrom, Write},
    os::raw,
};

use super::utils::{ReadExt, WriteExt};

const MASTER_HEADER_SIZE: usize = 22;
pub const PAGE_SIZE: usize = 4096;
const MAGIC: u32 = 0x54454144;
const VERSION: u16 = 1;

pub struct Paginator {
    path: String,
    file: File,
    version: u16,
    page_size: u32,
    total_pages: u32,
    free_list: u32,
    col_index: u32,
}

impl Paginator {
    pub fn new(path: &str, page_size: u32) -> Result<Self, Error> {
        let exists = std::path::Path::new(path).exists();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let mut paginator = Paginator {
            path: path.to_string(),
            file,
            version: VERSION,
            page_size,
            total_pages: 0,
            free_list: 0xffffffff,
            col_index: 0xffffffff,
        };
        if exists {
            paginator.read_header()?; // cargar estado existente
        } else {
            paginator.write_header()?; // inicializar desde cero
        }
        Ok(paginator)
    }

    fn read_header(&mut self) -> Result<(), Error> {
        let data = Paginator::read_chunk(&mut self.file, 0, MASTER_HEADER_SIZE)?;
        let mut cur = Cursor::new(&data);
        let magic = cur.read_u32_le()?;
        assert_eq!(magic, MAGIC);
        self.version = cur.read_u16_le()?;
        self.page_size = cur.read_u32_le()?;
        self.total_pages = cur.read_u32_le()?;
        self.free_list = cur.read_u32_le()?;
        self.col_index = cur.read_u32_le()?;
        Ok(())
    }

    fn write_header(&mut self) -> Result<(), Error> {
        let mut header = vec![0u8; MASTER_HEADER_SIZE];
        let mut cur = Cursor::new(&mut header);
        cur.write_u32_le(MAGIC)?;
        cur.write_u16_le(self.version)?;
        cur.write_u32_le(self.page_size)?;
        cur.write_u32_le(self.total_pages)?;
        cur.write_u32_le(self.free_list)?;
        cur.write_u32_le(self.col_index)?;

        debug_assert_eq!(
            cur.position() as usize,
            MASTER_HEADER_SIZE,
            "Header size mismatch:  {} bytes writen but MASTER_HEADER_SIZE is {}",
            cur.position(),
            MASTER_HEADER_SIZE
        );

        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&header)?;
        Ok(())
    }

    fn page_offset(&self, page_id: u32) -> u64 {
        (page_id * self.page_size) as u64
    }

    fn alloc_page_id(&mut self) -> Result<u32, Error> {
        if self.free_list == 0xFFFFFFFF {
            // No free pages, append one at the end
            self.total_pages += 1;
            Ok(self.total_pages)
        } else {
            let id = self.free_list;
            // +1 to omit the page_type byte
            let offset = self.page_offset(id) + 1;
            let bytes: [u8; 4] = Paginator::read_chunk(&mut self.file, offset, 4)?
                .as_slice()
                .try_into()
                .unwrap();
            self.free_list = u32::from_le_bytes(bytes);
            Ok(id)
        }
    }

    pub fn open(path: &str) -> Result<Self, Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .open(path)?; // TODO: improve error handling
        let header_bytes = Paginator::read_chunk(&mut file, 0, MASTER_HEADER_SIZE)?;
        let mut cur = Cursor::new(&header_bytes);
        assert_eq!(cur.read_u32_le()?, MAGIC);
        let dbfile = Paginator {
            path: path.to_string(),
            file,
            version: cur.read_u16_le()?,
            page_size: cur.read_u32_le()?,
            total_pages: cur.read_u32_le()?,
            free_list: cur.read_u32_le()?,
            col_index: cur.read_u32_le()?,
        };

        Ok(dbfile)
    }

    pub fn get_page(&mut self, num: u32) -> Result<Page, Error> {
        let page_id = self.page_offset(num);
        let raw_data = Paginator::read_chunk(&mut self.file, page_id, self.page_size as usize)?;
        let mut cur = Cursor::new(&raw_data);
        let mut page = Page {
            page_type: PageType::from_u8(cur.read_u8()?),
            next_page: cur.read_u32_le()?,
            data_len: cur.read_u16_le()?,
            payload: Vec::new(),
            page_size: self.page_size,
        };
        page.payload.resize(page.data_len as usize, 0);
        cur.read_exact(&mut page.payload)?;

        Ok(page)
    }

    pub fn create_page(&mut self, page_type: PageType, data: &[u8]) -> Result<(u32, Page), Error> {
        let page = Page {
            page_type,
            next_page: 0,
            data_len: data.len() as u16,
            payload: data.to_vec(),
            page_size: self.page_size,
        };
        let id = self.alloc_page_id()?;

        let _ = self.write_page(id, &page)?;
        self.write_header()?;
        Ok((id, page))
    }

    pub fn write_page(&mut self, num: u32, page: &Page) -> Result<(), Error> {
        self.file
            .seek(SeekFrom::Start((self.page_size * num) as u64))?;

        page.write_to(&mut self.file)?;
        self.file.flush()?;
        Ok(())
    }

    fn read_chunk(fd: &mut File, offset: u64, count: usize) -> Result<Vec<u8>, Error> {
        fd.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0; count];
        fd.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PageType {
    Index,
    Data,
    Overflow,
    Free,
}

impl PageType {
    fn from_u8(code: u8) -> PageType {
        match code {
            1 => PageType::Index,
            2 => PageType::Data,
            3 => PageType::Overflow,
            4 => PageType::Free,
            _ => PageType::Free,
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            PageType::Index => 1,
            PageType::Data => 2,
            PageType::Overflow => 3,
            PageType::Free => 4,
        }
    }
}

#[derive(Clone)]
pub struct Page {
    pub page_type: PageType,
    pub next_page: u32,
    data_len: u16,
    pub payload: Vec<u8>,
    page_size: u32,
}

impl Page {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.push(self.page_type.to_u8());

        buf.extend_from_slice(&self.next_page.to_le_bytes());
        buf.extend_from_slice(&self.data_len.to_le_bytes());

        // payload directo
        buf.extend_from_slice(&self.payload);

        buf
    }

    pub fn append(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let new_size = self.payload.len() + data.len() + 1 + 4 + 2;
        if new_size as u32 > self.page_size {
            return Err(Error::new(
                std::io::ErrorKind::FileTooLarge,
                "Data too large",
            ));
        }
        self.payload.append(&mut data.clone());
        self.data_len = self.payload.len() as u16;
        Ok(())
    }

    // zero-copy solution to write pages to the file
    pub fn write_to<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(&[self.page_type.to_u8()])?;
        w.write_all(&self.next_page.to_le_bytes())?;
        w.write_all(&self.data_len.to_le_bytes())?;
        w.write_all(&self.payload)?;

        // padding hasta completar page_size
        let written = (1 + 4 + 2 + self.payload.len()) as u32;
        let remaining = self.page_size.saturating_sub(written);
        w.write_all(&vec![0u8; remaining as usize])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_load() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let paginator = Paginator::new(&path, PAGE_SIZE as u32);
        assert!(paginator.is_ok());
        drop(paginator);
        let paginator = Paginator::open(&path);
        assert!(paginator.is_ok());
        let paginator = paginator.unwrap();
        assert_eq!(paginator.version, 1);
        assert_eq!(paginator.total_pages, 0);
    }

    #[test]
    fn test_pages_creation() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let paginator = Paginator::new(&path, PAGE_SIZE as u32);
        assert!(paginator.is_ok());
        let mut paginator = paginator.unwrap();
        let page = paginator.create_page(PageType::Data, "Hello world".as_bytes());
        assert!(page.is_ok());
        let page = paginator.create_page(PageType::Data, "Hello world".as_bytes());
        assert!(page.is_ok());
        drop(paginator);
        let paginator = Paginator::open(&path);
        assert!(paginator.is_ok());
        let mut paginator = paginator.unwrap();
        assert_eq!(paginator.version, 1);
        assert_eq!(paginator.total_pages, 2);
        let page = paginator.get_page(2);
        // assert!(page.is_ok());
        let page = page.unwrap();
        assert_eq!(page.page_type.to_u8(), PageType::Data.to_u8());
        assert_eq!(page.payload, "Hello world".as_bytes())
    }
}
