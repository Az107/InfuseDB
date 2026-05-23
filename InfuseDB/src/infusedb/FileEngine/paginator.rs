use std::{
    fs::File,
    io::{Cursor, Error, Read, Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
};

const MASTER_HEADER_SIZE: usize = 22;
const PAGE_SIZE: usize = 4096;
const MAGIC: u32 = 0x54454144;

trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8, Error> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }

    fn read_u16_le(&mut self) -> Result<u16, Error> {
        let mut b = [0u8; 2];
        self.read_exact(&mut b)?;
        Ok(u16::from_le_bytes(b))
    }

    fn read_u32_le(&mut self) -> Result<u32, Error> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(u32::from_le_bytes(b))
    }
}

impl ReadExt for Cursor<&Vec<u8>> {}

pub struct Paginator {
    path: String,
    _fd: File,
    version: u16,
    page_size: u32,
    total_pages: u32,
    free_list: u32,
    col_index: u32,
}

impl Paginator {
    pub fn open(path: String) -> Result<Self, Error> {
        let mut file = File::open(&path)?; // TODO: improve error handling
        let header_bytes = Paginator::read_chunk(&mut file, 0, MASTER_HEADER_SIZE)?;
        let mut cur = Cursor::new(&header_bytes);
        assert_eq!(cur.read_u32_le()?, MAGIC);
        let dbfile = Paginator {
            path,
            _fd: file,
            version: cur.read_u16_le()?,
            page_size: cur.read_u32_le()?,
            total_pages: cur.read_u32_le()?,
            free_list: cur.read_u32_le()?,
            col_index: cur.read_u32_le()?,
        };

        Ok(dbfile)
    }

    pub fn get_page(&mut self, num: u32) -> Result<Page, Error> {
        let raw_data = Paginator::read_chunk(
            &mut self._fd,
            (self.page_size * num) as u64,
            self.page_size as usize,
        )?;
        let mut cur = Cursor::new(&raw_data);
        let mut page = Page {
            page_type: PageType::from_u8(cur.read_u8()?),
            next_page: cur.read_u32_le()?,
            data_len: cur.read_u16_le()?,
            payload: Vec::new(),
        };
        let _c = cur.read_to_end(&mut page.payload)?;

        Ok(page)
    }

    pub fn write_page(&mut self, num: u32, page: Page) -> Result<(), Error> {
        self._fd
            .seek(SeekFrom::Start((self.page_size * num) as u64))?;
        page.write_to(&mut self._fd)?;
        Ok(())
    }

    fn read_chunk(fd: &mut File, offset: u64, count: usize) -> Result<Vec<u8>, Error> {
        fd.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0; count];
        fd.read_exact(&mut buf)?;
        Ok(buf)
    }
}

enum PageType {
    index,
    data,
    overflow,
    free,
}

impl PageType {
    fn from_u8(code: u8) -> PageType {
        match code {
            1 => PageType::index,
            2 => PageType::data,
            3 => PageType::overflow,
            4 => PageType::free,
            _ => PageType::free,
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            PageType::index => 1,
            PageType::data => 2,
            PageType::overflow => 3,
            PageType::free => 4,
        }
    }
}

pub struct Page {
    page_type: PageType,
    next_page: u32,
    data_len: u16,
    payload: Vec<u8>,
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

    // zero-copy solution to write pages to the file
    pub fn write_to<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(&[self.page_type.to_u8()])?;
        w.write_all(&self.next_page.to_le_bytes())?;
        w.write_all(&self.data_len.to_le_bytes())?;
        w.write_all(&self.payload)?;
        Ok(())
    }
}
