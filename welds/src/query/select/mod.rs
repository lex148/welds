use super::builder::QueryBuilder;
use super::clause::{DbParam, NextParam};
use super::helpers::{build_tail, build_where, join_sql_parts};
use crate::alias::TableAlias;
use crate::connection::Connection;
use crate::state::DbState;
use crate::table::{HasSchema, TableColumns, TableInfo};
use crate::writers::column::{ColumnWriter, DbColumnWriter};
use crate::writers::count::{CountWriter, DbCountWriter};
use crate::writers::limit_skip::DbLimitSkipWriter;
use anyhow::Result;
use sqlx::database::HasArguments;
use sqlx::IntoArguments;
use sqlx::Row;

// ******************************************************************************************
// This file contains all the stuff added onto the Querybuilder to allow it to run SELECTs
// ******************************************************************************************

impl<'schema, 'args, T, DB> QueryBuilder<'schema, T, DB>
where
    DB: sqlx::Database,
    T: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row> + HasSchema,
{
    /// Get a copy of the SQL that will be executed when this query runs
    pub fn to_sql(&self) -> String
    where
        <DB as HasArguments<'schema>>::Arguments: IntoArguments<'args, DB>,
        DB: DbParam + DbColumnWriter + DbLimitSkipWriter + DbCountWriter,
        <T as HasSchema>::Schema: TableInfo + TableColumns<DB>,
    {
        let mut args: Option<<DB as HasArguments>::Arguments> = None;
        let next_params = NextParam::new::<DB>();
        let wheres = self.wheres.as_slice();
        let exists_in = self.exist_ins.as_slice();
        let alias = TableAlias::new();
        let self_tablealias = alias.peek();

        join_sql_parts(&[
            build_head_select::<DB, <T as HasSchema>::Schema>(self_tablealias),
            build_where(&next_params, &alias, &mut args, wheres, exists_in),
            build_tail(self),
        ])
    }

    /// Executes a `select count(...) FROM ... `
    ///
    /// Counts the results of your query in the database.
    pub async fn count<'q, 'c, C>(&'q self, exec: &'c C) -> Result<u64>
    where
        'schema: 'args,
        C: 'schema,
        C: Connection<DB>,
        <DB as HasArguments<'schema>>::Arguments: IntoArguments<'args, DB>,
        DB: DbParam + DbColumnWriter + DbLimitSkipWriter + DbCountWriter,
        <T as HasSchema>::Schema: TableInfo + TableColumns<DB>,
        i64: sqlx::Type<DB> + for<'r> sqlx::Decode<'r, DB>,
        usize: sqlx::ColumnIndex<<DB as sqlx::Database>::Row>,
    {
        let mut args: Option<<DB as HasArguments>::Arguments> = Some(Default::default());
        let next_params = NextParam::new::<DB>();
        let wheres = self.wheres.as_slice();
        let exists_in = self.exist_ins.as_slice();
        let alias = TableAlias::new();
        let self_tablealias = alias.peek();

        let sql = join_sql_parts(&[
            build_head_count::<DB, <T as HasSchema>::Schema>(self_tablealias),
            build_where(&next_params, &alias, &mut args, wheres, exists_in),
            build_tail(self),
        ]);

        // lifetime hacks - Remove if you can
        // We know the use of sql and conn do not exceed the underlying call to fetch
        // sqlx if wants to hold the borrow for much longer than what is needed.
        // This hack prevents the borrow from exceeding the life of this call
        let sql_len = sql.len();
        let sqlp = sql.as_ptr();
        let sql_hack: &[u8] = unsafe { std::slice::from_raw_parts(sqlp, sql_len) };
        let sql: &str = std::str::from_utf8(sql_hack).unwrap();
        let exec_ptr: *const &C = &exec;
        let exec_hack: &mut C = unsafe { *(exec_ptr as *mut &mut C) };

        let rows = exec_hack.fetch_rows(sql, args.unwrap()).await?;
        let row = rows.get(0).unwrap();
        let count: i64 = row.try_get(0)?;
        Ok(count as u64)
    }

    /// Executes the query in the database returning the results
    pub async fn run<'q, 'c, C>(&'q self, exec: &'c C) -> Result<Vec<DbState<T>>>
    where
        'schema: 'args,
        C: 'schema,
        C: Connection<DB>,
        <DB as HasArguments<'schema>>::Arguments: IntoArguments<'args, DB>,
        DB: DbParam + DbColumnWriter + DbLimitSkipWriter,
        <T as HasSchema>::Schema: TableInfo + TableColumns<DB>,
    {
        let mut args: Option<<DB as HasArguments>::Arguments> = Some(Default::default());
        let next_params = NextParam::new::<DB>();
        let wheres = self.wheres.as_slice();
        let exists_in = self.exist_ins.as_slice();
        let alias = TableAlias::new();
        let self_tablealias = alias.peek();

        let sql = join_sql_parts(&[
            build_head_select::<DB, <T as HasSchema>::Schema>(self_tablealias),
            build_where(&next_params, &alias, &mut args, wheres, exists_in),
            build_tail(self),
        ]);

        // lifetime hacks - Remove if you can
        // We know the use of sql and conn do not exceed the underlying call to fetch
        // sqlx if wants to hold the borrow for much longer than what is needed.
        // This hack prevents the borrow from exceeding the life of this call
        let sql_len = sql.len();
        let sqlp = sql.as_ptr();
        let sql_hack: &[u8] = unsafe { std::slice::from_raw_parts(sqlp, sql_len) };
        let sql: &str = std::str::from_utf8(sql_hack).unwrap();
        let exec_ptr: *const &C = &exec;
        let exec_hack: &mut C = unsafe { *(exec_ptr as *mut &mut C) };

        let data = exec_hack
            .fetch_all(sql, args.unwrap())
            .await?
            .drain(..)
            .map(|d| DbState::db_loaded(d))
            .collect();

        Ok(data)
    }
}

fn build_head_select<DB, S>(tablealias: String) -> Option<String>
where
    DB: sqlx::Database + DbColumnWriter,
    S: TableInfo + TableColumns<DB>,
{
    let writer = ColumnWriter::new::<DB>();
    let mut head: Vec<&str> = Vec::default();
    head.push("SELECT");
    let cols_info = S::columns();
    let cols: Vec<_> = cols_info
        .iter()
        .map(|col| writer.write(&tablealias, col))
        .collect();
    let cols = cols.join(", ");
    head.push(&cols);
    head.push("FROM");
    let tn = S::identifier().join(".");
    let identifier = format!("{} {}", tn, tablealias);
    head.push(&identifier);
    Some(head.join(" "))
}

fn build_head_count<DB, S>(tablealias: String) -> Option<String>
where
    DB: sqlx::Database + DbColumnWriter + DbCountWriter,
    S: TableInfo + TableColumns<DB>,
{
    let tn = S::identifier().join(".");
    let identifier = format!("{} {}", tn, &tablealias);
    let cw = CountWriter::new::<DB>();
    let count_star = cw.count(Some(&tablealias), Some("*"));
    Some(format!("SELECT {} FROM {}", count_star, identifier))
}
