use bytes::{BufMut, BytesMut};
use postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use std::convert::TryInto;
use std::error::Error;

use crate::Bit;

impl<'a> FromSql<'a> for Bit<'a> {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Bit<'a>, Box<dyn Error + Sync + Send>> {
        Bit::from_sql(raw)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "bit"
    }
}

impl<'a> ToSql for Bit<'a> {
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let len = self.len;
        w.put_i32(len.try_into()?);

        for v in self.data {
            w.put_u8(*v);
        }

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "bit"
    }

    to_sql_checked!();
}

#[cfg(test)]
mod tests {
    use crate::Bit;
    use postgres::binary_copy::BinaryCopyInWriter;
    use postgres::types::Type;
    use postgres::{Client, NoTls};

    #[test]
    fn it_works() -> Result<(), postgres::Error> {
        let user = std::env::var("USER").unwrap();
        let mut client = Client::configure()
            .host("localhost")
            .dbname("pgvector_rust_test")
            .user(user.as_str())
            .connect(NoTls)?;

        client.execute("CREATE EXTENSION IF NOT EXISTS vector", &[])?;
        client.execute("DROP TABLE IF EXISTS postgres_bit_items", &[])?;
        client.execute(
            "CREATE TABLE postgres_bit_items (id bigserial PRIMARY KEY, embedding bit(8))",
            &[],
        )?;

        let vec = Bit::from_bytes(&[0b10101010]);
        let vec2 = Bit::from_bytes(&[0b01010101]);
        client.execute(
            "INSERT INTO postgres_bit_items (embedding) VALUES ($1), ($2), (NULL)",
            &[&vec, &vec2],
        )?;

        let query_vec = Bit::from_bytes(&[0b10101010]);
        let row = client.query_one(
            "SELECT embedding FROM postgres_bit_items ORDER BY embedding <~> $1 LIMIT 1",
            &[&query_vec],
        )?;
        let res_vec: Bit = row.get(0);
        assert_eq!(vec, res_vec);
        assert_eq!(8, res_vec.len());
        assert_eq!(&[0b10101010], res_vec.as_bytes());

        let null_row = client.query_one(
            "SELECT embedding FROM postgres_bit_items WHERE embedding IS NULL LIMIT 1",
            &[],
        )?;
        let null_res: Option<Bit> = null_row.get(0);
        assert!(null_res.is_none());

        // ensures binary format is correct
        let text_row = client.query_one(
            "SELECT embedding::text FROM postgres_bit_items ORDER BY id LIMIT 1",
            &[],
        )?;
        let text_res: String = text_row.get(0);
        assert_eq!("10101010", text_res);

        // copy
        let bit_type = Type::BIT;
        let writer = client
            .copy_in("COPY postgres_bit_items (embedding) FROM STDIN WITH (FORMAT BINARY)")?;
        let mut writer = BinaryCopyInWriter::new(writer, &[bit_type]);
        writer.write(&[&Bit::from_bytes(&[0b10101010])])?;
        writer.write(&[&Bit::from_bytes(&[0b01010101])])?;
        writer.finish()?;

        Ok(())
    }
}
