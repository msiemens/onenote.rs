use std::{fs::File, io::{BufReader, Read}};

use crate::utils::Result;


pub(crate) fn file_to_vec(file: &File) -> Result<Vec<u8>> {
    let size = file.metadata()?.len();
    let mut data = Vec::with_capacity(size as usize);

    let mut buf = BufReader::new(file);
    buf.read_to_end(&mut data)?;

    Ok(data)
}
