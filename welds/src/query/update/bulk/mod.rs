use crate::errors::Result;
use crate::model_traits::UniqueIdentifier;
use crate::model_traits::{HasSchema, TableColumns, TableInfo};
use crate::query::builder::QueryBuilder;
use crate::query::clause::wherein::WhereIn;
use crate::query::clause::ClauseAdder;
use crate::query::clause::ParamArgs;
use crate::query::clause::{AsFieldName, AsOptField};
use crate::query::helpers::{build_where, join_sql_parts};
use crate::writers::ColumnWriter;
use crate::writers::NextParam;
use crate::Client;
use crate::Syntax;
use std::marker::PhantomData;
use welds_connections::Param;

/// An un-executed Sql Update.
///
/// Build out a sql statement that will update the database in bulk
pub struct UpdateBuilder<T> {
    _t: PhantomData<T>,
    pub(crate) query_builder: QueryBuilder<T>,
    pub(crate) sets: Vec<Box<dyn ClauseAdder>>,
}

impl<T> UpdateBuilder<T>
where
    T: Send + HasSchema,
{
    pub(crate) fn new(query_builder: QueryBuilder<T>) -> Self {
        Self {
            _t: Default::default(),
            sets: Vec::default(),
            query_builder,
        }
    }

    /// Sets the value from the lambda in the database
    ///
    /// ```
    /// use welds::prelude::*;
    ///
    /// #[derive(Debug, Default, WeldsModel)]
    /// #[welds(table = "things")]
    /// struct Thing {
    ///     #[welds(primary_key)]
    ///     pub id: i32,
    ///     pub foo: i32,
    /// }
    ///
    /// async fn example(db: &dyn Client) -> welds::errors::Result<()> {
    ///     Thing::all().set(|x| x.foo, 42).run(db).await?;
    ///     // [UPDATE things SET foo = ?]   (?=42)
    ///     Ok(())
    /// }
    ///
    pub fn set<V, FIELD>(
        mut self,
        lam: impl Fn(<T as HasSchema>::Schema) -> FIELD,
        value: impl Into<V>,
    ) -> Self
    where
        <T as HasSchema>::Schema: Default,
        FIELD: AsFieldName<V>,
        V: 'static + Sync + Send + Clone + Param,
    {
        let val: V = value.into();
        let field = lam(Default::default());
        let col_raw = field.colname().to_string();
        self.sets.push(Box::new(SetColVal { col_raw, val }));
        self
    }

    /// Sets a custom [`ClauseAdder`] value from the lambda in the database
    /// This is funcionally the same as set, but you can provide any thing that impl [`ClauseAdder`]
    ///
    /// ```
    /// use welds::prelude::*;
    ///
    /// #[derive(Debug, Default, WeldsModel)]
    /// #[welds(table = "things")]
    /// struct Thing {
    ///     #[welds(primary_key)]
    ///     pub id: i32,
    ///     pub foo: i32,
    /// }
    ///
    /// async fn example(db: &dyn Client) -> welds::errors::Result<()> {
    ///     Thing::all().set_col(|x| x.foo.equal(42) ).run(db).await?;
    ///     // [UPDATE things SET foo = ?]   (?=42)
    ///     Ok(())
    /// }
    ///
    /// ```
    pub fn set_col(mut self, lam: impl Fn(<T as HasSchema>::Schema) -> Box<dyn ClauseAdder>) -> Self
    where
        <T as HasSchema>::Schema: Default,
        //V: 'static + Sync + Send + Clone + Param,
    {
        self.sets.push(lam(Default::default()));
        self
    }

    /// Nulls out the value from the lambda in the database
    pub fn set_null<V, FIELD>(mut self, lam: impl Fn(<T as HasSchema>::Schema) -> FIELD) -> Self
    where
        <T as HasSchema>::Schema: Default,
        FIELD: AsFieldName<V> + AsOptField,
        V: 'static + Sync + Send + Clone + Param,
    {
        let field = lam(Default::default());
        let col_raw = field.colname().to_string();
        self.sets.push(Box::new(SetColNull { col_raw }));
        self
    }

    /// Get a copy of the SQL that will be executed when this query runs
    pub fn to_sql(&self, syntax: Syntax) -> String
    where
        <T as HasSchema>::Schema: UniqueIdentifier + TableInfo + TableColumns,
    {
        let mut w_in = WhereIn::new(&self.query_builder);

        self.sql_internal(syntax, &mut w_in, &mut None)
    }

    fn sql_internal<'s, 'w, 'args, 'p>(
        &'s self,
        syntax: Syntax,
        w_in: &'w mut WhereIn<T>,
        args: &'args mut Option<ParamArgs<'p>>,
    ) -> String
    where
        'w: 'p,
        's: 'p,
        <T as HasSchema>::Schema: UniqueIdentifier + TableInfo + TableColumns,
    {
        let next_params = NextParam::new(syntax);
        let sets = self.sets.as_slice();
        let alias = <T as HasSchema>::Schema::identifier().join(".");

        join_sql_parts(&[
            build_head::<<T as HasSchema>::Schema>(syntax, &next_params, &alias, args, sets),
            build_where_update(
                syntax,
                w_in,
                &next_params,
                &alias,
                args,
                &self.query_builder,
            ),
        ])
    }

    /// Executes the query in the database Bulk updating the values
    pub async fn run<'s, 'c>(&'s self, client: &'c dyn Client) -> Result<()>
    where
        <T as HasSchema>::Schema: UniqueIdentifier + TableInfo + TableColumns,
    {
        let syntax = client.syntax();
        let mut args: Option<ParamArgs> = Some(Vec::default());
        let mut w_in = WhereIn::new(&self.query_builder);
        let sql = self.sql_internal(syntax, &mut w_in, &mut args);
        let args = args.unwrap();
        let _results = client.execute(&sql, &args).await?;

        Ok(())
    }
}

fn build_head<'s, 'args, 'p, S>(
    syntax: Syntax,
    next_params: &NextParam,
    alias: &str,
    args: &'args mut Option<ParamArgs<'p>>,
    sets: &'s [Box<dyn ClauseAdder>],
) -> Option<String>
where
    's: 'p,
    S: TableInfo + TableColumns,
{
    let tn = S::identifier().join(".");

    let mut set_parts: Vec<String> = Vec::default();

    for clause in sets {
        if let Some(args) = args {
            clause.bind(args);
        }
        if let Some(p) = clause.clause(syntax, alias, next_params) {
            set_parts.push(p);
        }
    }
    let set_sql = set_parts.join(", ");

    Some(format!("UPDATE {tn} SET {sets}", tn = tn, sets = set_sql))
}

pub struct SetColVal<T> {
    pub col_raw: String,
    pub val: T,
}

impl<T> ClauseAdder for SetColVal<T>
where
    T: Clone + Send + Sync + Param,
{
    /// Add the argument to the list of Arguments to send to the database
    fn bind<'lam, 'args, 'p>(&'lam self, args: &'args mut ParamArgs<'p>)
    where
        'lam: 'p,
    {
        args.push(&self.val);
    }

    /// Returns the SQL snipit for this clause
    fn clause(&self, syntax: Syntax, _alias: &str, next_params: &NextParam) -> Option<String> {
        let colname = ColumnWriter::new(syntax).excape(&self.col_raw);
        let sql = format!("{}={}", colname, next_params.next());
        Some(sql)
    }
}

pub struct SetColNull {
    pub col_raw: String,
}

impl ClauseAdder for SetColNull {
    /// Add the argument to the list of Arguments to send to the database
    fn bind<'lam, 'args, 'p>(&'lam self, _args: &'args mut ParamArgs<'p>)
    where
        'lam: 'p,
    {
        // no args added
    }

    /// Returns the SQL snipit for this clause
    fn clause(&self, syntax: Syntax, _alias: &str, _next_params: &NextParam) -> Option<String> {
        let colname = ColumnWriter::new(syntax).excape(&self.col_raw);
        let sql = format!("{}=NULL", colname);
        Some(sql)
    }
}

pub(crate) fn build_where_update<'q, 'w, 'args, 'p, T>(
    syntax: Syntax,
    w_in: &'w mut WhereIn<T>,
    next_params: &NextParam,
    alias: &str,
    args: &'args mut Option<ParamArgs<'p>>,
    qb: &'q QueryBuilder<T>,
) -> Option<String>
where
    'q: 'p,
    'w: 'p,
    T: HasSchema,
    <T as HasSchema>::Schema: UniqueIdentifier + TableInfo + TableColumns,
{
    // If we have a limit, we need to wrap the wheres in an IN clause
    // this is to limit the number of row to that will be updated
    if qb.limit.is_none() {
        let wheres = qb.wheres.as_slice();
        let exists_in = qb.exist_ins.as_slice();
        return build_where(syntax, next_params, alias, wheres, args, exists_in);
    }

    let mut where_sql: Vec<String> = Vec::default();
    if let Some(args) = args {
        w_in.bind(args);
    }

    // use fulltable name for alias when updating
    let tableparts = T::Schema::identifier();
    let outer_tablealias = tableparts.join(".");

    if let Some(p) = w_in.clause(syntax, &outer_tablealias, next_params) {
        where_sql.push(p);
    }

    if where_sql.is_empty() {
        return None;
    }
    Some(format!("WHERE ( {} )", where_sql.join(" AND ")))
}

#[cfg(test)]
mod tests;
