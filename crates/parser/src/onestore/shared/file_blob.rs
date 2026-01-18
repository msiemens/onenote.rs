use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Default, Eq, PartialEq, PartialOrd)]
pub struct FileBlob(Rc<Vec<u8>>);

impl Debug for FileBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();
        let first_32 = self
            .0
            .iter()
            .take(32)
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        let last_32 = self
            .0
            .iter()
            .rev()
            .take(32)
            .rev()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        write!(
            f,
            "FileBlob [ {} ... {}; {:?} KiB ]",
            first_32,
            last_32,
            len / 1024
        )
    }
}

impl FileBlob {
    pub fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for FileBlob {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

impl<'a> From<&'a [u8]> for FileBlob {
    fn from(value: &'a [u8]) -> Self {
        Self(value.to_vec().into())
    }
}
