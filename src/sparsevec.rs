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

    /// TODO
    pub fn to_vec(&self) -> Vec<f32> {
        let mut vec = vec![0.0; self.dim];
        for (i, v) in self.indices.iter().zip(&self.values) {
            vec[*i as usize] = *v;
        }
        vec
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

#[cfg(test)]
mod tests {
    use crate::SparseVec;

    #[test]
    fn test_to_vec() {
        let vec = SparseVec::new(5, vec![0, 2, 4], vec![1.0, 2.0, 3.0]);
        assert_eq!(vec![1.0, 0.0, 2.0, 0.0, 3.0], vec.to_vec());
    }
}
