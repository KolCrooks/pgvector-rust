use bytes::{BufMut, BytesMut};
use postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use std::convert::TryInto;
use std::error::Error;

use crate::HalfVec;

impl<'a> FromSql<'a> for HalfVec {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<HalfVec, Box<dyn Error + Sync + Send>> {
        HalfVec::from_sql(raw)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "halfvec"
    }
}

impl ToSql for HalfVec {
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let dim = self.0.len();
        w.put_u16(dim.try_into()?);
        w.put_u16(0);

        for v in &self.0 {
            w.put(&v.to_be_bytes()[..]);
        }

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "halfvec"
    }

    to_sql_checked!();
}

#[cfg(test)]
mod tests {
    use crate::HalfVec;
    use half::f16;
    use postgres::binary_copy::BinaryCopyInWriter;
    use postgres::types::{Kind, Type};
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
        client.execute("DROP TABLE IF EXISTS postgres_half_items", &[])?;
        client.execute(
            "CREATE TABLE postgres_half_items (id bigserial PRIMARY KEY, embedding halfvec(3))",
            &[],
        )?;

        let vec = HalfVec::from(vec![
            f16::from_f32(1.0),
            f16::from_f32(2.0),
            f16::from_f32(3.0),
        ]);
        let vec2 = HalfVec::from(vec![
            f16::from_f32(4.0),
            f16::from_f32(5.0),
            f16::from_f32(6.0),
        ]);
        client.execute(
            "INSERT INTO postgres_half_items (embedding) VALUES ($1), ($2), (NULL)",
            &[&vec, &vec2],
        )?;

        let query_vec = HalfVec::from(vec![
            f16::from_f32(3.0),
            f16::from_f32(1.0),
            f16::from_f32(2.0),
        ]);
        let row = client.query_one(
            "SELECT embedding FROM postgres_half_items ORDER BY embedding <-> $1 LIMIT 1",
            &[&query_vec],
        )?;
        let res_vec: HalfVec = row.get(0);
        assert_eq!(vec, res_vec);
        assert_eq!(
            vec![f16::from_f32(1.0), f16::from_f32(2.0), f16::from_f32(3.0)],
            res_vec.to_vec()
        );

        let empty_vec = HalfVec::from(vec![]);
        let empty_res = client.execute(
            "INSERT INTO postgres_half_items (embedding) VALUES ($1)",
            &[&empty_vec],
        );
        assert!(empty_res.is_err());
        assert!(empty_res
            .unwrap_err()
            .to_string()
            .contains("halfvec must have at least 1 dimension"));

        let null_row = client.query_one(
            "SELECT embedding FROM postgres_half_items WHERE embedding IS NULL LIMIT 1",
            &[],
        )?;
        let null_res: Option<HalfVec> = null_row.get(0);
        assert!(null_res.is_none());

        // ensures binary format is correct
        let text_row = client.query_one(
            "SELECT embedding::text FROM postgres_half_items ORDER BY id LIMIT 1",
            &[],
        )?;
        let text_res: String = text_row.get(0);
        assert_eq!("[1,2,3]", text_res);

        // copy
        let halfvec_type = get_type(&mut client, "halfvec")?;
        let writer = client
            .copy_in("COPY postgres_half_items (embedding) FROM STDIN WITH (FORMAT BINARY)")?;
        let mut writer = BinaryCopyInWriter::new(writer, &[halfvec_type]);
        writer.write(&[&HalfVec::from(vec![
            f16::from_f32(1.0),
            f16::from_f32(2.0),
            f16::from_f32(3.0),
        ])])?;
        writer.write(&[&HalfVec::from(vec![
            f16::from_f32(4.0),
            f16::from_f32(5.0),
            f16::from_f32(6.0),
        ])])?;
        writer.finish()?;

        Ok(())
    }

    fn get_type(client: &mut Client, name: &str) -> Result<Type, postgres::Error> {
        let row = client.query_one("SELECT pg_type.oid, nspname AS schema FROM pg_type INNER JOIN pg_namespace ON pg_namespace.oid = pg_type.typnamespace WHERE typname = $1", &[&name])?;
        Ok(Type::new(
            name.into(),
            row.get("oid"),
            Kind::Simple,
            row.get("schema"),
        ))
    }
}
