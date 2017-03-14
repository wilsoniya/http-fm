use std::env::home_dir;
use std::fs::create_dir_all;
use std::path::PathBuf;

use chrono::offset::utc::UTC;
use rusqlite::Connection;
use rusqlite::types::ToSql;
use rusqlite::Error as SQLErrorType;

use data::CodePath;
use utils::HFMError;

static DB_FPATH: &'static str = "/home/wilsoniya/.config/http_fm/db";
static DB_VERSION: i64 = 1;
static CONFIG_REL_DPATH: &'static str = ".config/http_fm";
static DB_FNAME: &'static str = "db";

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
static SELECT_ALL_CODE_PATHS_SQL: &'static str = "
SELECT code, path, expiration, hits
FROM code_path";
static DELETE_CODE_PATH_SQL: &'static str = "
DELETE from code_path
where code = ?";


#[derive(Debug)]
pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn open(maybe_db_fpath: Option<&str>) -> Result<DB, HFMError> {
        let db_fpath = match maybe_db_fpath {
            Some(db_fpath) => db_fpath.to_owned(),
            None => {
                let db_pathbuf = DB::get_db_fpath() ?;
                let db_fpath = db_pathbuf.to_str().ok_or(HFMError::EnvError(
                    "Can't resolve db file path.".to_owned())) ?;
                db_fpath.to_owned()
            }
        };
        let conn = Connection::open(db_fpath) ?;
        let db = DB { conn: conn };

        db.create_tables() ?;
        db.upgrade_schema() ?;

        Ok(db)
    }

    fn get_db_fpath() -> Result<PathBuf, HFMError> {
        if let Some(home_dir) = home_dir() {
            // case: home dir exists
            let config_dpath = home_dir.join(PathBuf::from(CONFIG_REL_DPATH));

            if !config_dpath.exists() {
                create_dir_all(&config_dpath) ?;
            }

            if config_dpath.is_dir() {
                Ok(config_dpath.join(PathBuf::from(DB_FNAME)))
            } else {
                // case: config path is somehow a file; bail
                Err(HFMError::EnvError(
                    format!("Config path points to a file: {:?}", config_dpath).to_owned()))
            }
        } else {
            Err(HFMError::EnvError(
                "Unable to resolve home directory to open config.".to_owned()))
        }

    }

    fn open_in_memory() -> Result<DB, HFMError> {
        let conn = Connection::open_in_memory() ?;
        let db = DB { conn: conn };

        db.create_tables() ?;
        db.upgrade_schema() ?;

        Ok(db)
    }

    fn create_tables(&self) -> Result<(), HFMError> {
        self.conn.execute(SCHEMA_VERSION_DDL, &[]) ?;
        self.conn.execute(CODE_PATH_DDL, &[]) ?;

        Ok(())
    }

    fn upgrade_schema(&self) -> Result<(), HFMError> {
        // TODO: make this actually do upgrades
        self.conn.execute(DELETE_SCHEMA_VERSION_SQL, &[]) ?;
        self.conn.execute(SET_SCHEMA_VERSION_SQL, &[&DB_VERSION]) ?;

        Ok(())
    }

    fn get_schema_version(&self) -> Result<Option<i64>, HFMError> {
        let ret = self.conn.prepare(GET_SCHEMA_VERSION_SQL)
        .and_then(|mut stmt| {
            stmt.query_row(&[], |row| row.get("version"))
        }) ?;

        Ok(ret)
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

    pub fn get_all_code_paths(&self) -> Result<Vec<CodePath>, HFMError> {
        let code_paths = self.conn.prepare(SELECT_ALL_CODE_PATHS_SQL)
        .and_then(|mut stmt| {
            stmt.query_map(&[], |row| {
                let hits: i64 = row.get("hits");
                let path: String = row.get("path");
                CodePath {
                    code: row.get("code"),
                    path: PathBuf::from(path),
                    expiration: row.get("expiration"),
                    hits: hits as u64,
                }
            })
            .map(|code_paths| {
                code_paths
                .filter_map(|maybe_code_path| maybe_code_path.ok())
                .collect::<Vec<CodePath>>()
            })
        }) ?;

        Ok(code_paths)
    }

    pub fn delete_code_path(&self, code: &str) -> Result<usize, HFMError> {
        let num_rows = self.conn.execute(DELETE_CODE_PATH_SQL, &[&code]) ?;
        Ok(num_rows as usize)
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
    fn test_open_twice() {
        let temp_file = &mktemp::Temp::new_file().unwrap();

        let db = DB::open(Some(temp_file.as_ref().to_str().unwrap()));
        assert!(db.is_ok());

        let db = DB::open(Some(temp_file.as_ref().to_str().unwrap()));
        assert!(db.is_ok());
    }

    #[test]
    fn test_insert_code_path() {
        let db = DB::open_in_memory().unwrap();

        let result = db.insert_code_path("fart", "/path", None);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_duplicate_code_path() {
        let db = DB::open_in_memory().unwrap();

        let result = db.insert_code_path("fart", "/path", None);
        println!("{:?}", result);
        let result = db.insert_code_path("fart", "/path2", None);
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_path() {
        let db = DB::open_in_memory().unwrap();

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
    fn test_delete_code_path() {
        let db = DB::open_in_memory().unwrap();
        db.insert_code_path("fart", "/path", None);

        let result = db.delete_code_path("fart");
        assert_eq!(result.unwrap(), 1);

        let result2 = db.delete_code_path("fart");
        assert_eq!(result2.unwrap(), 0);
    }

    #[test]
    fn test_increment_hit_count() {
        let db = DB::open_in_memory().unwrap();

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

    #[test]
    fn test_get_schema_version() {
        let res = DB::open_in_memory();
        println!("res {:?}", res);
        let db = DB::open_in_memory().unwrap();
//      let schema_version = db.get_schema_version();
//      assert_eq!(schema_version.unwrap().unwrap(), 1);
    }
}
