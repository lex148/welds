use crate::errors::{Result, WeldsError};
use crate::model_traits::{HasSchema, TableColumns, TableInfo, WriteToArgs};
use crate::query::clause::ParamArgs;
use crate::writers::ColumnWriter;
use crate::writers::NextParam;
use welds_connections::Client;
use welds_connections::Row;

pub async fn update_one<T, C>(obj: &mut T, client: &C) -> Result<()>
where
    T: WriteToArgs + HasSchema,
    <T as HasSchema>::Schema: TableInfo + TableColumns,
    T: TryFrom<Row>,
    WeldsError: From<<T as TryFrom<Row>>::Error>,
    C: Client,
{
    let syntax = client.syntax();
    let mut args: ParamArgs = Vec::default();
    let col_writer = ColumnWriter::new(syntax);
    let next_params = NextParam::new(syntax);

    let identifier = <<T as HasSchema>::Schema>::identifier().join(".");
    let columns = <<T as HasSchema>::Schema as TableColumns>::columns();
    let pks = <<T as HasSchema>::Schema as TableColumns>::primary_keys();
    if pks.is_empty() {
        return Err(WeldsError::NoPrimaryKey);
    }
    let mut sets = Vec::default();

    for col in columns {
        if !pks.contains(&col) {
            obj.bind(col.name(), &mut args)?;
            let p = next_params.next();
            let colname = col_writer.excape(col.name());
            sets.push(format!("{}={}", colname, p));
        }
    }
    if sets.is_empty() {
        return Ok(());
    }
    let mut wheres = Vec::default();
    for col in pks {
        obj.bind(col.name(), &mut args)?;
        let p = next_params.next();
        let colname = col_writer.excape(col.name());
        wheres.push(format!("{}={}", colname, p));
    }

    let sets = sets.join(", ");
    let wheres = wheres.join(" AND ");

    let sql = format!("UPDATE {} SET {} where {}", identifier, sets, wheres);

    client.execute(&sql, &args).await?;

    Ok(())
}

#[cfg(test)]
mod tests;
