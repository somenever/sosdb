use sosdb::{Object, Value, Database, DatabaseLoadError};

fn main() -> Result<(), DatabaseLoadError> {
    let mut database = Database::new("test.sosdb".into());
    database.load()?;
    dbg!(database);

    Ok(())
}
