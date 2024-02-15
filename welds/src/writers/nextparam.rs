use crate::Syntax;
use std::sync::{Arc, Mutex};

pub struct NextParam {
    i: Arc<Mutex<usize>>,
    db_next: fn(usize) -> String,
    db_max: fn() -> u32,
}

impl NextParam {
    pub fn new(syntax: Syntax) -> Self {
        let next_fn_ptr = match syntax {
            #[cfg(feature = "mysql")]
            Syntax::Mysql => MySql::next,
            #[cfg(feature = "postgres")]
            Syntax::Postgres => Postgres::next,
            #[cfg(feature = "mssql")]
            Syntax::Mssql => Mssql::next,
            #[cfg(feature = "sqlite")]
            Syntax::Sqlite => Sqlite::next,
        };

        let max_fn_ptr = match syntax {
            #[cfg(feature = "mysql")]
            Syntax::Mysql => MySql::max_params,
            #[cfg(feature = "postgres")]
            Syntax::Postgres => Postgres::max_params,
            #[cfg(feature = "mssql")]
            Syntax::Mssql => Mssql::max_params,
            #[cfg(feature = "sqlite")]
            Syntax::Sqlite => Sqlite::max_params,
        };

        Self {
            i: Arc::new(Mutex::new(1)),
            db_next: next_fn_ptr,
            db_max: max_fn_ptr,
        }
    }

    pub fn next(&self) -> String {
        let lock = self.i.clone();
        let mut i = lock.lock().unwrap();
        let p = (self.db_next)(*i);
        *i += 1;
        p
    }

    pub fn max_params(&self) -> u32 {
        (self.db_max)()
    }
}

#[cfg(feature = "postgres")]
struct Postgres;
#[cfg(feature = "postgres")]
impl Postgres {
    fn next(i: usize) -> String {
        format!("${}", i)
    }
    fn max_params() -> u32 {
        65535
    }
}

#[cfg(feature = "sqlite")]
struct Sqlite;
#[cfg(feature = "sqlite")]
impl Sqlite {
    fn next(_i: usize) -> String {
        "?".to_string()
    }
    fn max_params() -> u32 {
        999
    }
}

#[cfg(feature = "mssql")]
struct Mssql;
#[cfg(feature = "mssql")]
impl Mssql {
    fn next(i: usize) -> String {
        format!("@p{}", i)
    }
    fn max_params() -> u32 {
        60
        //2100
    }
}

#[cfg(feature = "mysql")]
struct MySql;
#[cfg(feature = "mysql")]
impl MySql {
    fn next(_i: usize) -> String {
        "?".to_string()
    }
    fn max_params() -> u32 {
        64000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pg_should_use_dollar_signs_with_numbers() {
        let p = NextParam::new(Syntax::Postgres);
        assert_eq!(p.next(), "$1");
        assert_eq!(p.next(), "$2");
        assert_eq!(p.next(), "$3");
        assert_eq!(p.next(), "$4");
    }

    #[test]
    fn mssql_should_use_at_signs_with_numbers() {
        let p = NextParam::new(Syntax::Mssql);
        assert_eq!(p.next(), "@p1");
        assert_eq!(p.next(), "@p2");
        assert_eq!(p.next(), "@p3");
        assert_eq!(p.next(), "@p4");
    }

    #[test]
    fn mysql_should_use_question_marks() {
        let p = NextParam::new(Syntax::Mysql);
        assert_eq!(p.next(), "?");
        assert_eq!(p.next(), "?");
        assert_eq!(p.next(), "?");
        assert_eq!(p.next(), "?");
    }

    #[test]
    fn sqlite_should_use_question_marks() {
        let p = NextParam::new(Syntax::Sqlite);
        assert_eq!(p.next(), "?");
        assert_eq!(p.next(), "?");
        assert_eq!(p.next(), "?");
        assert_eq!(p.next(), "?");
    }
}
