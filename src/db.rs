use chrono::offset::utc::UTC;
use rusqlite::Connection;
use rusqlite::types::ToSql;

use utils::HFMError;

static DB_FPATH: &'static str = "/home/wilsoniya/.config/http_fm/db";
static DB_VERSION: i32 = 1;

static SCHEMA_VERSION_DDL: &'static str =
"CREATE TABLE IF NOT EXISTS schema_version( version INTEGER )";
static GET_SCHEMA_VERSION_SQL: &'static str =
"SELECT version FROM schema_version";
static DELETE_SCHEMA_VERSION_SQL: &'static str =
"DELETE FROM schema_version";
static SET_SCHEMA_VERSION_SQL: &'static str =
"INSERT INTO schema_version VALUES (?1)";


static CODE_DPATH_DDL: &'static str = "
CREATE TABLE IF NOT EXISTS code_dpath (
    code TEXT PRIMARY KEY,
    dpath TEXT NOT NULL,
    expiration INTEGER,
    hits INTEGER
)";
static INSERT_CODE_DPATH_SQL: &'static str = "
INSERT INTO code_dpath (code, dpath, expiration, hits)
VALUES (?1, ?2, ?3, ?4)";
static SELECT_DPATH_BY_CODE_SQL: &'static str = "
SELECT dpath, expiration
FROM code_dpath
WHERE code = ?1";



struct DB {
    conn: Connection,
}

impl DB {
    pub fn open(db_fpath: Option<&str>) -> Result<DB, HFMError> {
        let db_fpath = db_fpath.unwrap_or(DB_FPATH);
        let conn = Connection::open(db_fpath) ?;
        let db = DB { conn: conn };
        Ok(db)
    }

    fn open_in_memory() -> Result<DB, HFMError> {
        let conn = Connection::open_in_memory() ?;
        let db = DB { conn: conn };
        Ok(db)
    }

    pub fn create_tables(&self) -> Result<(), HFMError> {
        self.conn.execute(SCHEMA_VERSION_DDL, &[]) ?;
        self.conn.execute(DELETE_SCHEMA_VERSION_SQL, &[]) ?;
        self.conn.execute(SET_SCHEMA_VERSION_SQL, &[&DB_VERSION]) ?;
        self.conn.execute(CODE_DPATH_DDL, &[]) ?;

        Ok(())
    }

    pub fn insert_code_dpath(&self, code: &str, dpath: &str,
                             expiration: Option<i64>) -> Result<(), HFMError> {
        self.conn.execute(
            INSERT_CODE_DPATH_SQL, &[&code, &dpath, &expiration, &0]) ?;

        Ok(())
    }

    pub fn get_dpath(&self, code: &str) -> Result<Option<String>, HFMError> {
        let dpath: Option<String> = self.conn.prepare(SELECT_DPATH_BY_CODE_SQL)
        .and_then(|mut stmt| {
            stmt.query_row(&[&code], |row| (row.get(0), row.get(1)))
        })
        .map(|(dpath, expiration): (String, Option<i64>)| {
            match expiration {
                None => Some(dpath.to_owned()),
                Some(expiration) => {
                    match UTC::now().timestamp() < expiration {
                        true => Some(dpath.to_owned()),
                        false => None
                    }
                }
            }
        }) ?;

        Ok(dpath)
    }
}

#[cfg(test)]
mod test {
    extern crate mktemp;

    use chrono::offset::utc::UTC;

    use db::DB;

    #[test]
    fn test_open() {
        let temp_file = &mktemp::Temp::new_file().unwrap();
        let db = DB::open(Some(temp_file.as_ref().to_str().unwrap()));
        assert!(db.is_ok());
    }

    #[test]
    fn test_create_tables() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());
    }

    #[test]
    fn test_insert_code_dpath() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let result = db.insert_code_dpath("fart", "/path", None);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_dpath() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let code = "foobar";
        let dpath = "/path";

        let result = db.insert_code_dpath(code, dpath, None);
        assert!(result.is_ok());

        let maybe_result = db.get_dpath(code);
        assert!(maybe_result.is_ok());
        let result = maybe_result.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), dpath);
    }

    #[test]
    fn test_get_dpath_expired() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let code = "foobar";
        let dpath = "/path";
        let expiration = Some(UTC::now().timestamp());

        let result = db.insert_code_dpath(code, dpath, expiration);
        assert!(result.is_ok());

        let maybe_result = db.get_dpath(code);
        assert!(maybe_result.is_ok());
        let result = maybe_result.unwrap();
        assert!(result.is_none());
    }
}
