use granitedb::db::GraniteDB;

fn main() -> anyhow::Result<()> {
    let mut db = GraniteDB::new().build()?;
    db.put("foo".to_string(), "bar".to_string());
    print!("{:?}", db.get("foo"));
    Ok(())
}
