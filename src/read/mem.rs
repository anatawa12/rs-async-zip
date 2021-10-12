// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-zip/blob/main/LICENSE)

//! A module for reading ZIP file entries concurrently from an in-memory buffer.

use crate::error::{Result, ZipError};
use crate::read::{ZipEntry, ZipEntryReader, CompressionReader};

use std::io::{Cursor, SeekFrom};

use tokio::io::{AsyncSeekExt};

/// The type returned as an entry reader within this concurrent module.
pub type ConcurrentReader<'b, 'a> = ZipEntryReader<'b, Cursor<&'a [u8]>>;

/// A reader which acts concurrently over an in-memory buffer.
pub struct ZipFileReader<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) entries: Vec<ZipEntry>,
}

impl<'a> ZipFileReader<'a> {
    /// Constructs a new ZIP file reader from an in-memory buffer.
    pub async fn new(data: &'a [u8]) -> Result<ZipFileReader<'a>> {
        let entries = crate::read::seek::read_cd(&mut Cursor::new(data)).await?;
        Ok(ZipFileReader { data, entries })
    }

    crate::read::reader_entry_impl!();

    /// Opens an entry at the provided index for reading.
    pub async fn entry_reader<'b>(&'b mut self, index: usize) -> Result<ConcurrentReader<'b, 'a>> {
        let entry = self.entries.get(index).ok_or(ZipError::EntryIndexOutOfBounds)?;
        let mut cursor = Cursor::new(self.data.clone());

        cursor.seek(SeekFrom::Start(entry.data_offset())).await?;
        let reader = CompressionReader::from_reader(entry.compression(), cursor);

        Ok(ZipEntryReader { entry, reader })
    }
}