/// A sparse vector.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseVec {
    pub(crate) dim: usize,
    pub(crate) indices: Vec<i32>,
    pub(crate) values: Vec<f32>,
}

impl SparseVec {
    /// TODO
    pub fn new(dim: usize, indices: Vec<i32>, values: Vec<f32>) -> SparseVec {
        SparseVec {
            dim,
            indices,
            values,
        }
    }

    #[cfg(any(feature = "postgres"))]
    pub(crate) fn from_sql(
        buf: &[u8],
    ) -> Result<SparseVec, Box<dyn std::error::Error + Sync + Send>> {
        let dim = i32::from_be_bytes(buf[0..4].try_into()?) as usize;
        let nnz = i32::from_be_bytes(buf[4..8].try_into()?) as usize;
        let unused = i32::from_be_bytes(buf[8..12].try_into()?);
        if unused != 0 {
            return Err("expected unused to be 0".into());
        }

        let mut indices = Vec::with_capacity(nnz);
        for i in 0..nnz {
            let s = 12 + 4 * i;
            indices.push(i32::from_be_bytes(buf[s..s + 4].try_into()?) - 1);
        }

        let mut values = Vec::with_capacity(nnz);
        for i in 0..nnz {
            let s = 12 + 4 * nnz + 4 * i;
            values.push(f32::from_be_bytes(buf[s..s + 4].try_into()?));
        }

        Ok(SparseVec {
            dim,
            indices,
            values,
        })
    }
}
