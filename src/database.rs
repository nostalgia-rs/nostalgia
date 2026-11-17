use polars::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::env::current_dir;
use rand::prelude::*;
use rand::rng;


fn get_lazyframe() -> PolarsResult<LazyFrame> {
    println!("Running DB!");

    let schema= Schema::from_iter(vec![
        Field::new("category".into(), DataType::String),
        Field::new("subcategory".into(), DataType::String),
        Field::new("index".into(), DataType::String),
        Field::new("name".into(), DataType::String),
        Field::new("dark_threshold".into(), DataType::String),
        // Field::new("dark_threshold".into(), DataType::Float64),
        Field::new("background_color".into(), DataType::String),
        Field::new("foreground_color".into(), DataType::String),
        Field::new("type".into(), DataType::String),
        Field::new("second_type".into(), DataType::String),
    ]);
    let schema_ref: SchemaRef = Arc::new(schema);

    println!("foo");
    println!("CWD: {:?}", current_dir()?);

    let database_path = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".nostalgia/database.csv"))
        .ok_or_else(|| PolarsError::ComputeError("HOME environment variable is not set".into()))?;
    let database_path = database_path
        .to_str()
        .ok_or_else(|| PolarsError::ComputeError("Database path is not valid UTF-8".into()))?;

    LazyCsvReader::new(database_path.into())
        .with_has_header(true)
        .with_schema(Some(schema_ref.clone()))
        .finish()//?.collect()
}

pub fn get_random_row() -> PolarsResult<DataFrame> {
    let lf = get_lazyframe()?;
    let count_df = lf.clone()
        .select([len().alias("total_rows")])
        .collect()?;
    let total_rows = count_df
        .column("total_rows")?
        .u32()?
        .get(0)
        .ok_or_else(|| PolarsError::NoData("row count query returned no rows".into()))?
        as usize;

    if total_rows == 0 {
        return Ok(DataFrame::empty());
    }

    let offset = rng().random_range(0..total_rows) as i64;
    lf.slice(offset, 1).collect()
}
