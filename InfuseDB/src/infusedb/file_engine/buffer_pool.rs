use std::fs;
use std::io::{Error, ErrorKind};

use super::paginator::{PAGE_SIZE, Page, PageType, Paginator};

use super::lru::LRU;

pub struct BufferPool {
    cache: LRU<Page>,
    paginator: Paginator,
}

impl BufferPool {
    pub fn new(path: &str) -> Result<Self, Error> {
        let paginator = if fs::exists(path).unwrap() {
            Paginator::open(path)?
        } else {
            Paginator::new(path, (PAGE_SIZE * 2) as u32)?
        };
        Ok(BufferPool {
            cache: LRU::new(10),
            paginator,
        })
    }

    fn flush(&mut self, page_id: u32) -> Result<(), Error> {
        let page = self.cache.get(page_id).unwrap();
        self.paginator.write_page(page_id, page)?;
        self.cache.unpin(page_id)?;

        Ok(())
    }

    pub fn flush_all(&mut self) -> Result<(), Error> {
        while let Some(page_id) = self.cache.get_evict_candidate() {
            self.flush(page_id)?;
            self.cache.evict()?;
        }
        Ok(())
    }

    fn load_page_into_cache(&mut self, page_id: u32) -> Option<()> {
        let page = self.paginator.get_page(page_id).ok()?;

        let mut attempts = 0;
        loop {
            match self.cache.set(page_id, page.clone()) {
                Ok(_) => return Some(()),
                Err(err) => {
                    if attempts >= 10 {
                        return None;
                    }
                    match err.kind() {
                        ErrorKind::OutOfMemory => {
                            let candidate = self.cache.get_evict_candidate()?;
                            self.flush(candidate).ok()?;
                            attempts += 1;
                        }
                        _ => return None,
                    }
                }
            }
        }
    }

    pub fn get_page(&mut self, page_id: u32) -> Option<&Page> {
        if !self.cache.has(page_id) {
            self.load_page_into_cache(page_id)?;
        }

        Some(self.cache.get(page_id)?)
    }

    pub fn get_mut_page(&mut self, page_id: u32) -> Option<&mut Page> {
        if !self.cache.has(page_id) {
            self.load_page_into_cache(page_id)?;
        }
        self.cache.pin(page_id);
        Some(self.cache.get_mut(page_id)?)
    }

    pub fn create_page(&mut self, page_type: PageType) -> Result<u32, Error> {
        let (id, _page) = self.paginator.create_page(page_type, &Vec::new())?;
        self.load_page_into_cache(id).ok_or(Error::new(
            ErrorKind::OutOfMemory,
            "Error loading page into cache",
        ))?;
        Ok(id)
    }

    pub fn update_page(&mut self, page_id: u32, data: Vec<u8>) {
        self.cache.write(page_id, |page| {
            page.payload = data;
        });
        let _ = self.cache.pin(page_id);
    }
}
