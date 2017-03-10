use rusqlite::Connection;

use utils::HFMError;

static DB_FPATH: &'static str = "/home/wilsoniya/.config/http_fm/db";
static DB_VERSION: u64 = 1;

static SCHEMA_VERSION_DDL: &'static str =
"CREATE TABLE schema_version( version INTEGER )";
static GET_SCHEMA_VERSION_SQL: &'static str =
"SELECT version FROM schema_version";


struct DB {
    conn: Connection,
}

impl DB {
    pub fn open() -> Result<DB, HFMError> {
        let conn = Connection::open(DB_FPATH)?;
        let db = DB { conn: conn };
        Ok(db)
    }

    pub fn create_tables(&self) {
    }
}

#[cfg(test)]
mod test {
    use db::DB;

    #[test]
    fn test_function_name() {
        let db = DB::open();
    }
}
