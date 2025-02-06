use super::{AsFieldName, ClauseColVal, ClauseColValEqual};
use std::marker::PhantomData;
use welds_connections::Param;

pub struct Text<T> {
    col: String,
    field: String,
    _t: PhantomData<T>,
}

impl<T> AsFieldName<T> for Text<T> {
    fn colname(&self) -> &str {
        self.col.as_str()
    }
    fn fieldname(&self) -> &str {
        self.field.as_str()
    }
}

impl<T> Text<T>
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

    pub fn equal(self, v: impl Into<T>) -> Box<ClauseColValEqual<T>>
    where
        T: Param,
    {
        let cv = ClauseColValEqual::<T> {
            null_clause: false,
            not_clause: false,
            col: self.col,
            operator: "=",
            val: Some(v.into()),
        };
        Box::new(cv)
    }

    pub fn not_equal(self, v: impl Into<T>) -> Box<ClauseColVal<T>>
    where
        T: Param,
    {
        let cv = ClauseColVal::<T> {
            null_clause: false,
            not_clause: true,
            col: self.col,
            operator: "!=",
            val: Some(v.into()),
        };
        Box::new(cv)
    }

    pub fn like(self, v: impl Into<T>) -> Box<ClauseColVal<T>>
    where
        T: Param,
    {
        let cv = ClauseColVal::<T> {
            null_clause: false,
            not_clause: false,
            col: self.col,
            operator: "like",
            val: Some(v.into()),
        };
        Box::new(cv)
    }

    pub fn not_like(self, v: impl Into<T>) -> Box<ClauseColVal<T>>
    where
        T: Param,
    {
        let cv = ClauseColVal::<T> {
            null_clause: false,
            not_clause: true,
            col: self.col,
            operator: "not like",
            val: Some(v.into()),
        };
        Box::new(cv)
    }

    pub fn ilike(self, v: impl Into<T>) -> Box<ClauseColVal<T>>
    where
        T: Param,
    {
        let cv = ClauseColVal::<T> {
            null_clause: false,
            not_clause: false,
            col: self.col,
            operator: "ilike",
            val: Some(v.into()),
        };
        Box::new(cv)
    }

    pub fn not_ilike(self, v: impl Into<T>) -> Box<ClauseColVal<T>>
    where
        T: Param,
    {
        let cv = ClauseColVal::<T> {
            null_clause: false,
            not_clause: true,
            col: self.col,
            operator: "not ilike",
            val: Some(v.into()),
        };
        Box::new(cv)
    }
}
