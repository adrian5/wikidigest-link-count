/*
Reads an input source into borrowed (read_into) or owned (read) chunks of desired size. This opens
the door to parallel processing.
*/
use std::io::{Read, Result};

pub struct ChunkedReader<T: Read> {
    source: T,
    remainder: Vec<u8>,
    exhausted: bool,
}

impl<T: Read> ChunkedReader<T> {
    pub fn new(source: T) -> Self {
        Self {
            source,
            remainder: Vec::new(),
            exhausted: false,
        }
    }

    #[allow(dead_code)]
    pub fn read_into(&mut self, dest: &mut String, target_size: usize) -> Result<bool> {
        if self.exhausted {
            return Ok(false);
        }

        assert!(dest.capacity() >= target_size); // Sanity check

        // Insert prior remainder before reading in new data
        // Remainders can end with chopped multi-byte chars, so don't UTF-8 validate!
        if !self.remainder.is_empty() {
            unsafe {
                let dest = dest.as_mut_vec();
                dest.set_len(self.remainder.len());
                dest.copy_from_slice(&self.remainder);
            }
        }

        // Read into buffer until full (or out of input data)
        let mut bytes_read_total = self.remainder.len();
        unsafe {
            let dest = dest.as_mut_vec();
            dest.set_len(target_size); // Restore to full target size
            while bytes_read_total < target_size {
                let bytes_read = self.source.read(&mut dest[bytes_read_total..])?;
                if bytes_read == 0 {
                    // Assume final read
                    dest.truncate(bytes_read_total);
                    self.exhausted = true;
                    return Ok(false); // Signal final read
                }
                bytes_read_total += bytes_read;
            }
        }

        // Split at last newline, store right-hand side
        self.remainder.clear();
        if let Some(cutoff) = dest.rfind('\n') {
            self.remainder.extend_from_slice(&dest.as_bytes()[cutoff..]);
            dest.truncate(cutoff);
        }

        Ok(true) // Not final read
    }

    #[allow(dead_code)]
    pub fn read(&mut self, target_size: usize) -> Result<Option<String>> {
        if self.exhausted {
            return Ok(None);
        }

        let mut chunk = String::with_capacity(target_size);

        // Insert prior remainder before reading in new data
        // Remainders can end with chopped multi-byte chars, so don't UTF-8 validate!
        if !self.remainder.is_empty() {
            unsafe {
                let chunk = chunk.as_mut_vec();
                chunk.set_len(self.remainder.len());
                chunk.copy_from_slice(&self.remainder);
            }
        }

        // Read into buffer until full (or out of input data)
        let mut bytes_read_total = self.remainder.len();
        unsafe {
            let chunk = chunk.as_mut_vec();
            chunk.set_len(target_size); // Restore to full target size
            while bytes_read_total < target_size {
                let bytes_read = self.source.read(&mut chunk[bytes_read_total..])?;
                if bytes_read == 0 {
                    // Assume final read
                    chunk.truncate(bytes_read_total);
                    self.exhausted = true;
                    break;
                }
                bytes_read_total += bytes_read;
            }
        }

        if !self.exhausted {
            // Split at last newline, keep right-hand side for next chunk
            self.remainder.clear();
            if let Some(cutoff) = chunk.rfind('\n') {
                self.remainder
                    .extend_from_slice(&chunk.as_bytes()[cutoff..]);
                chunk.truncate(cutoff);
            }
        }

        Ok(Some(chunk))
    }
}
