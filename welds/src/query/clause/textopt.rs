use super::{AsFieldName, AsOptField, ClauseAdder, ClauseColVal};
use crate::query::optional::HasSomeNone;
use crate::query::optional::Optional;
use std::marker::PhantomData;
use welds_connections::Param;

pub struct TextOpt<T> {
    col: String,
    field: String,
    _t: PhantomData<T>,
}

impl<T> AsFieldName<T> for TextOpt<T> {
    fn colname(&self) -> &str {
        self.col.as_str()
    }
    fn fieldname(&self) -> &str {
        self.field.as_str()
    }
}

impl<T> AsOptField for TextOpt<T> {}

impl<T> TextOpt<T>
where
    T: 'static + Clone + Send + Sync,
{
    pub fn new(col: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            col: col.into(),
            field: field.into(),
            _t: Default::default(),
        }
    }

    pub fn equal(self, v: impl Into<Optional<T>>) -> Box<dyn ClauseAdder>
    where
        T: Param,
    {
        let opt = v.into();
        let is_none = opt.is_none();
        let val: Option<T> = opt.into();

        let cv = ClauseColVal::<T> {
            null_clause: is_none,
            not_clause: false,
            col: self.col,
            operator: "=",
            val,
        };
        Box::new(cv)
    }

    pub fn not_equal(self, v: impl Into<Optional<T>>) -> Box<dyn ClauseAdder>
    where
        T: Param,
    {
        let opt = v.into();
        let is_none = opt.is_none();
        let val: Option<T> = opt.into();

        let cv = ClauseColVal::<T> {
            null_clause: is_none,
            not_clause: true,
            col: self.col,
            operator: "!=",
            val,
        };
        Box::new(cv)
    }

    pub fn like(self, v: impl Into<Optional<T>>) -> Box<dyn ClauseAdder>
    where
        T: Param,
    {
        let opt = v.into();
        let val: Option<T> = opt.into();

        let cv = ClauseColVal::<T> {
            null_clause: val.is_none(),
            not_clause: false,
            col: self.col,
            operator: "like",
            val,
        };
        Box::new(cv)
    }

    pub fn not_like(self, v: impl Into<Optional<T>>) -> Box<dyn ClauseAdder>
    where
        T: Param,
    {
        let opt = v.into();
        let val: Option<T> = opt.into();

        let cv = ClauseColVal::<T> {
            null_clause: val.is_none(),
            not_clause: true,
            col: self.col,
            operator: "not like",
            val,
        };
        Box::new(cv)
    }

    pub fn ilike(self, v: impl Into<Optional<T>>) -> Box<dyn ClauseAdder>
    where
        T: Param,
    {
        let opt = v.into();
        let val: Option<T> = opt.into();

        let cv = ClauseColVal::<T> {
            null_clause: val.is_none(),
            not_clause: false,
            col: self.col,
            operator: "ilike",
            val,
        };
        Box::new(cv)
    }

    pub fn not_ilike(self, v: impl Into<Optional<T>>) -> Box<dyn ClauseAdder>
    where
        T: Param,
    {
        let opt = v.into();
        let val: Option<T> = opt.into();

        let cv = ClauseColVal::<T> {
            null_clause: val.is_none(),
            not_clause: true,
            col: self.col,
            operator: "not ilike",
            val,
        };
        Box::new(cv)
    }
}
