use polars::prelude::*;
use rand::prelude::*;
use rand::rng;
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;

fn get_lazyframe() -> PolarsResult<LazyFrame> {
    println!("Running DB!");

    let schema = Schema::from_iter(vec![
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

    println!("CWD: {}", current_dir()?.display());
    println!("HOME: {:?}", std::env::var_os("HOME"));

    let database_path = std::env::var_os("HOME").map_or_else(
        || {
            Err(PolarsError::ComputeError(
                "HOME environment variable is not set".into(),
            ))
        },
        |home| Ok(PathBuf::from(home).join(".nostalgia/**/*.csv")),
    )?;
    let database_path = database_path
        .to_str()
        .ok_or_else(|| PolarsError::ComputeError("Database path is not valid UTF-8".into()))?;

    LazyCsvReader::new(database_path.into())
        .with_glob(true)
        .with_has_header(true)
        .with_schema(Some(schema_ref))
        .finish() //?.collect()
}

fn filter_values(lf: LazyFrame, column: &str, values: &[String]) -> LazyFrame {
    if values.iter().any(|v| v.to_lowercase() == "all") {
        return lf;
    }

    let value_filter = values
        .iter()
        .filter(|value| value.as_str() != "all")
        .map(|value| col(column).eq(lit(value.clone())))
        .reduce(Expr::or)
        .unwrap_or_else(|| lit(false));

    lf.filter(value_filter)
}

/// Retrieves a random row matching the configured filters.
///
/// # Errors
///
/// Returns an error if the database cannot be read or the result cannot be collected.
pub fn get_random_row(
    categories: &[String],
    subcategories: &[String],
    names: &[String],
    types: &[String],
) -> PolarsResult<DataFrame> {
    let category_filtered_lf = filter_values(get_lazyframe()?, "category", categories);
    let subcategory_filtered_lf = filter_values(category_filtered_lf, "subcategory", subcategories);
    let name_filtered_lf = filter_values(subcategory_filtered_lf, "name", names);
    let type_filtered_lf = filter_values(name_filtered_lf, "type", types);
    let count_df = type_filtered_lf
        .clone()
        .select([len().alias("total_rows")])
        .collect()?;
    let total_rows = count_df
        .column("total_rows")?
        .u32()?
        .get(0)
        .ok_or_else(|| PolarsError::NoData("row count query returned no rows".into()))
        .and_then(|count| {
            usize::try_from(count)
                .map_err(|_| PolarsError::ComputeError("row count is too large".into()))
        })?;

    if total_rows == 0 {
        return Ok(DataFrame::empty());
    }

    let offset = i64::try_from(rng().random_range(0..total_rows))
        .map_err(|_| PolarsError::ComputeError("row count exceeds supported offset".into()))?;
    type_filtered_lf.slice(offset, 1).collect()
}

#[cfg(test)]
mod tests {
    use super::filter_values;
    use polars::df;
    use polars::prelude::{IntoLazy, PolarsResult};

    #[test]
    fn filters_rows_to_all_configured_values() -> PolarsResult<()> {
        let frame = df!(
            "category" => ["pokemon", "plants", "pokemon", "pokemon", "pokemon"],
            "subcategory" => ["fire", "fire", "water", "fire", "fire"],
            "name" => ["charmander", "charmander", "charmander", "squirtle", "charmander"],
            "type" => ["Fire", "Fire", "Water", "Water", "Water"],
            "index" => ["1", "2", "3", "4", "5"],
        )?;

        let filtered = filter_values(frame.lazy(), "category", &["pokemon".to_owned()]);
        let filtered = filter_values(filtered, "subcategory", &["fire".to_owned()]);
        let filtered = filter_values(filtered, "name", &["charmander".to_owned()]);
        let filtered = filter_values(filtered, "type", &["Fire".to_owned()]).collect()?;

        assert_eq!(filtered.column("index")?.str()?.get(0), Some("1"));
        assert_eq!(filtered.height(), 1);
        Ok(())
    }

    #[test]
    fn all_categories_disables_filtering() -> PolarsResult<()> {
        let frame = df!("category" => ["pokemon", "plants"])?;
        let categories = vec!["ALL".to_owned()];

        let filtered = filter_values(frame.lazy(), "category", &categories).collect()?;

        assert_eq!(filtered.height(), 2);
        Ok(())
    }
}
