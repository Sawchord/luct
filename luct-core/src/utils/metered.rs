use std::io::{Read, Result};

pub(crate) struct MeteredRead<'a, R>(&'a mut R, usize);

impl<'a, R> MeteredRead<'a, R> {
    pub fn new(read: &'a mut R) -> Self {
        Self(read, 0)
    }

    pub fn get_meter(&self) -> usize {
        self.1
    }
}

impl<R: Read> Read for MeteredRead<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = self.0.read(buf)?;
        self.1 += read;
        Ok(read)
    }
}
