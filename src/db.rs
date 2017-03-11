use std::path::PathBuf;

use chrono::offset::utc::UTC;
use rusqlite::Connection;
use rusqlite::types::ToSql;

use data::CodePath;
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


static CODE_PATH_DDL: &'static str = "
CREATE TABLE IF NOT EXISTS code_path (
    code TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    expiration INTEGER,
    hits INTEGER NOT NULL
)";
static INSERT_CODE_PATH_SQL: &'static str = "
INSERT INTO code_path (code, path, expiration, hits)
VALUES (?1, ?2, ?3, ?4)";
static SELECT_PATH_BY_CODE_SQL: &'static str = "
SELECT code, path, expiration, hits
FROM code_path
WHERE code = ?1";
static UPDATE_CODE_PATH_HITS_SQL: &'static str = "
UPDATE code_path
SET hits = ?1
WHERE code = ?2";


pub struct DB {
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
        self.conn.execute(CODE_PATH_DDL, &[]) ?;

        Ok(())
    }

    pub fn insert_code_path(&self, code: &str, path: &str,
                             expiration: Option<i64>) -> Result<(), HFMError> {
        self.conn.execute(
            INSERT_CODE_PATH_SQL, &[&code, &path, &expiration, &0]) ?;

        Ok(())
    }

    pub fn get_code_path(&self, code: &str
                         ) -> Result<Option<CodePath>, HFMError> {
        let ret: Option<CodePath> = self.conn.prepare(SELECT_PATH_BY_CODE_SQL)
        .and_then(|mut stmt| {
            stmt.query_row(&[&code], |row| {
                let code: String = row.get("code");
                let path: String = row.get("path");
                let expiration = row.get("expiration");
                let hits: i64 = row.get("hits");

                CodePath {
                    code: code,
                    path: PathBuf::from(path),
                    expiration: expiration,
                    hits: hits as u64,
                }
            })
        })
        .map(|codepath| {
            match codepath.expiration {
                None => Some(codepath),
                Some(expiration) => {
                    match UTC::now().timestamp() < expiration {
                        true => Some(codepath),
                        false => None
                    }
                }
            }
        }) ?;

        Ok(ret)
    }

    pub fn increment_hit_count(&self, code: &str) -> Option<i64> {
        self.get_code_path(code).ok()
        .and_then(|maybe_code_path: Option<CodePath>| {
            // case: successfully queried for code_path
            maybe_code_path
            .and_then(|code_path| {
                // case: successfully resolved code_path
                let cur_hits = (code_path.hits + 1) as i64;
                self.conn.execute(UPDATE_CODE_PATH_HITS_SQL,
                                  &[&cur_hits, &code]).ok()
                .map(|_| cur_hits)
            })
        })
    }
}

#[cfg(test)]
mod test {
    extern crate mktemp;

    use std::path::PathBuf;

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
    fn test_insert_code_path() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let result = db.insert_code_path("fart", "/path", None);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_duplicate_code_path() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let result = db.insert_code_path("fart", "/path", None);
        println!("{:?}", result);
        let result = db.insert_code_path("fart", "/path2", None);
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_path() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let code = "foobar";
        let path = "/path";

        let result = db.insert_code_path(code, path, None);
        assert!(result.is_ok());

        let maybe_result = db.get_code_path(code);
        assert!(maybe_result.is_ok());
        let result = maybe_result.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, PathBuf::from(path));
    }

    #[test]
    fn test_get_path_expired() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let code = "foobar";
        let path = "/path";
        let expiration = Some(UTC::now().timestamp());

        let result = db.insert_code_path(code, path, expiration);
        assert!(result.is_ok());

        let maybe_result = db.get_code_path(code);
        assert!(maybe_result.is_ok());
        let result = maybe_result.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_increment_hit_count() {
        let db = DB::open_in_memory().unwrap();
        assert!(db.create_tables().is_ok());

        let result = db.insert_code_path("fart", "/path", None);
        println!("{:?}", result);
        assert!(result.is_ok());

        let maybe_new_hit_count = db.increment_hit_count("fart");
        assert!(maybe_new_hit_count.is_some());
        assert_eq!(maybe_new_hit_count.unwrap(), 1);

        let maybe_new_hit_count = db.increment_hit_count("fart");
        assert!(maybe_new_hit_count.is_some());
        assert_eq!(maybe_new_hit_count.unwrap(), 2);
    }
}
