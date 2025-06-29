use std::io::{Read, Result};

pub(crate) struct MeteredRead<R>(R, usize);

impl<R> MeteredRead<R> {
    pub fn new(read: R) -> Self {
        Self(read, 0)
    }

    pub fn get_meter(&self) -> usize {
        self.1
    }
}

impl<R: Read> Read for MeteredRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = self.0.read(buf)?;
        self.1 += read;
        Ok(read)
    }
}
