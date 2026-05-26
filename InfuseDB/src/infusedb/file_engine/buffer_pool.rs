use std::io::Error;

use super::paginator::{Page, PageType, Paginator};

use super::lru::LRU;

pub struct BufferPool {
    cache: LRU<Page>,
    paginator: Paginator,
}

impl BufferPool {
    pub fn new(path: &str) -> Result<Self, Error> {
        Ok(BufferPool {
            cache: LRU::new(10),
            paginator: Paginator::open(path)?,
        })
    }

    fn flush(&mut self, page_id: u32) -> Result<(), &'static str> {
        let page = self.cache.get(page_id).unwrap();
        self.paginator
            .write_page(page_id, page)
            .map_err(|_| "Error writing page")?;
        self.cache.unpin(page_id)?;

        Ok(())
    }

    pub fn flush_all(&mut self) -> Result<(), &'static str> {
        while let Some(page_id) = self.cache.get_evict_candidate() {
            self.flush(page_id)?;
            self.cache.evict()?;
        }
        Ok(())
    }

    pub fn get_page(&mut self, page_id: u32) -> Option<&Page> {
        if !self.cache.has(page_id) {
            let page = self.paginator.get_page(page_id).ok()?;
            let r = self.cache.set(page_id, page);
            if r.is_err() {
                let candidate = self.cache.get_evict_candidate()?;
                self.flush(candidate).ok()?; //If can allocate a page return None ?
            }
        }

        Some(self.cache.get(page_id)?)
    }

    pub fn create_page(&mut self, page_type: PageType) -> Result<u32, &'static str> {
        let (id, page) = self
            .paginator
            .create_page(page_type, &Vec::new())
            .map_err(|_| "Error creating page")?;
        self.cache.set(id, page)?;
        Ok(id)
    }

    pub fn update_page(&mut self, page_id: u32, data: Vec<u8>) {
        self.cache.write(page_id, |page| {
            page.payload = data;
        });
        let _ = self.cache.pin(page_id);
    }
}
