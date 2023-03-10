pub struct ColArg(pub String, pub String);
use super::column::ColumnWriter;
use crate::table::Column;

type Sql = String;

pub(crate) struct InsertWriter {
    write: fn(identifier: &str, &[ColArg], &[Column]) -> Sql,
}

impl InsertWriter {
    pub fn new<DB: DbInsertWriter>() -> Self {
        Self { write: DB::write }
    }

    pub fn write(&self, identifier: &str, colargs: &[ColArg], columns: &[Column]) -> Sql {
        (self.write)(identifier, colargs, columns)
    }
}

pub trait DbInsertWriter {
    fn write(identifier: &str, colargs: &[ColArg], columns: &[Column]) -> Sql;
}

#[cfg(feature = "postgres")]
impl DbInsertWriter for sqlx::Postgres {
    fn write(identifier: &str, colargs: &[ColArg], columns: &[Column]) -> Sql {
        let cols: Vec<_> = colargs.iter().map(|x| x.0.as_str()).collect();
        let args: Vec<_> = colargs.iter().map(|x| x.1.as_str()).collect();
        let col_group = cols.join(", ");
        let arg_group = args.join(", ");
        format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            identifier, col_group, arg_group
        )
    }
}

#[cfg(feature = "sqlite")]
impl DbInsertWriter for sqlx::Sqlite {
    fn write(identifier: &str, colargs: &[ColArg], columns: &[Column]) -> Sql {
        todo!();
    }
}

#[cfg(feature = "mysql")]
impl DbInsertWriter for sqlx::MySql {
    fn write(identifier: &str, colargs: &[ColArg], columns: &[Column]) -> Sql {
        todo!();
    }
}

#[cfg(feature = "mssql")]
impl DbInsertWriter for sqlx::Mssql {
    fn write(identifier: &str, colargs: &[ColArg], columns: &[Column]) -> Sql {
        let cols: Vec<_> = colargs.iter().map(|x| x.0.as_str()).collect();
        let args: Vec<_> = colargs.iter().map(|x| x.1.as_str()).collect();
        let col_group = cols.join(", ");
        let arg_group = args.join(", ");

        // write the column select that will be returned
        let col_write = ColumnWriter::new::<sqlx::Mssql>();
        let return_col: Vec<String> = columns
            .iter()
            .map(|c| col_write.write_with_prefix("Inserted", c))
            .collect();
        let outputs = return_col.join(", ");

        format!(
            "INSERT INTO {} ({}) OUTPUT {} VALUES ({})",
            identifier, col_group, outputs, arg_group
        )
    }
}
