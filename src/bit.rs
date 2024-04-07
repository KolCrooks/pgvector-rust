/// A bit string.
#[derive(Clone, Debug, PartialEq)]
pub struct Bit<'a> {
    pub(crate) len: usize,
    pub(crate) data: &'a [u8],
}

impl<'a> Bit<'a> {
    /// TODO
    pub fn from_bytes(data: &'a [u8]) -> Bit {
        Bit {
            // TODO check for overflow
            len: data.len() * 8,
            data,
        }
    }

    /// TODO
    pub fn len(&self) -> usize {
        self.len
    }

    /// TODO
    pub fn as_bytes(&self) -> &'a [u8] {
        self.data
    }

    #[cfg(any(feature = "postgres"))]
    pub(crate) fn from_sql(buf: &[u8]) -> Result<Bit, Box<dyn std::error::Error + Sync + Send>> {
        let len = i32::from_be_bytes(buf[0..4].try_into()?) as usize;
        let data = &buf[4..4 + len / 8];

        Ok(Bit { len, data })
    }
}

#[cfg(test)]
mod tests {
    use crate::Bit;

    #[test]
    fn test_as_bytes() {
        let vec = Bit::from_bytes(&[0b00000000, 0b11111111]);
        assert_eq!(16, vec.len());
        assert_eq!(&[0b00000000, 0b11111111], vec.as_bytes());
    }
}
