use polars::prelude::*;
use rand::prelude::*;
use rand::rng;
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;

fn get_lazyframe() -> PolarsResult<LazyFrame> {
    tracing::debug!("Running DB!");

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

    tracing::debug!("CWD: {}", current_dir()?.display());
    tracing::debug!("HOME: {:?}", std::env::var_os("HOME"));

    let database_path = database_glob(std::env::var_os("HOME"))?;

    LazyCsvReader::new(database_path.as_str().into())
        .with_glob(true)
        .with_has_header(true)
        .with_schema(Some(schema_ref))
        .finish() //?.collect()
}

fn database_glob(home: Option<std::ffi::OsString>) -> PolarsResult<String> {
    let home = home
        .ok_or_else(|| PolarsError::ComputeError("HOME environment variable is not set".into()))?;
    PathBuf::from(home)
        .join(".nostalgia/**/*.csv")
        .into_os_string()
        .into_string()
        .map_err(|_| PolarsError::ComputeError("Database path is not valid UTF-8".into()))
}

/// Loads all rows from the user's database CSV files without applying filters.
///
/// # Errors
///
/// Returns an error if a CSV file cannot be read or collected.
pub fn get_all_rows() -> PolarsResult<DataFrame> {
    get_lazyframe()?.collect()
}

/// Counts database rows grouped by category.
///
/// # Errors
///
/// Returns an error if the category column is missing or the aggregation fails.
pub fn get_category_counts(database: &DataFrame) -> PolarsResult<DataFrame> {
    database
        .clone()
        .lazy()
        .group_by([col("category")])
        .agg([len().alias("count")])
        .collect()
}

fn row_count(count_df: &DataFrame) -> PolarsResult<usize> {
    if count_df.height() == 0 {
        return Err(PolarsError::NoData(
            "row count query returned no rows".into(),
        ));
    }

    let count = count_df
        .column("total_rows")?
        .u32()?
        .get(0)
        .ok_or_else(|| PolarsError::NoData("row count query returned no rows".into()))?;
    row_count_value(count)
}

fn row_count_value<T: TryInto<usize>>(count: T) -> PolarsResult<usize> {
    count
        .try_into()
        .map_err(|_| PolarsError::ComputeError("row count is too large".into()))
}

fn row_offset<T: TryInto<i64>>(count: T) -> PolarsResult<i64> {
    count
        .try_into()
        .map_err(|_| PolarsError::ComputeError("row count exceeds supported offset".into()))
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
    let total_rows = row_count(&count_df)?;

    if total_rows == 0 {
        return Ok(DataFrame::empty());
    }

    let offset = row_offset(rng().random_range(0..total_rows))?;
    type_filtered_lf.slice(offset, 1).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        database_glob, filter_values, get_category_counts, row_count, row_count_value, row_offset,
    };
    use polars::df;
    use polars::prelude::{DataFrame, IntoLazy, PolarsResult};

    #[test]
    fn filters_rows_to_all_configured_values() -> PolarsResult<()> {
        let frame = df!(
            "category" => ["pokemon", "plants", "pokemon", "pokemon", "pokemon"],
            "subcategory" => ["fire", "fire", "water", "fire", "fire"],
            "name" => ["charmander", "charmander", "charmander", "squirtle", "charmander"],
            "type" => ["Fire", "Fire", "Water", "Water", "Water"],
            "index" => ["1", "2", "3", "4", "5"],
        )
        .unwrap();

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

    #[test]
    fn groups_database_rows_and_counts_them_by_category() -> PolarsResult<()> {
        let frame = df!("category" => ["pokemon", "pokemon", "plants"])?;

        let counts = get_category_counts(&frame)?;

        for (category, expected_count) in [("pokemon", 2), ("plants", 1)] {
            let mut category_count = None;
            for index in 0..counts.height() {
                if counts.column("category")?.get(index)?.str_value().as_ref() == category {
                    category_count = counts.column("count")?.u32()?.get(index);
                    break;
                }
            }
            assert_eq!(category_count, Some(expected_count));
        }
        Ok(())
    }

    #[test]
    fn empty_filter_matches_no_rows() -> PolarsResult<()> {
        let frame = df!("category" => ["pokemon", "plants"])?;

        let filtered = filter_values(frame.lazy(), "category", &[]).collect()?;

        assert_eq!(filtered.height(), 0);
        Ok(())
    }

    #[test]
    fn resolves_the_database_glob_or_reports_invalid_home_paths() {
        let home = std::env::temp_dir().into_os_string();
        assert!(
            database_glob(Some(home))
                .unwrap()
                .ends_with(".nostalgia/**/*.csv")
        );
        assert!(database_glob(None).is_err());

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;

            assert!(database_glob(Some(std::ffi::OsString::from_vec(vec![0xff]))).is_err());
        }
    }

    #[test]
    fn reports_row_count_schema_and_empty_result_errors() -> PolarsResult<()> {
        assert!(row_count(&DataFrame::empty()).is_err());
        assert!(row_count(&df!("other_column" => [1])?).is_err());
        assert!(row_count(&df!("total_rows" => ["not a count"])?).is_err());
        assert!(row_count(&df!("total_rows" => Vec::<u32>::new())?).is_err());
        assert!(row_count(&df!("total_rows" => [None::<u32>])?).is_err());
        assert!(row_count_value(u128::MAX).is_err());
        assert!(row_offset(u128::MAX).is_err());
        assert!(df!("left" => [1, 2], "right" => [1]).is_err());
        Ok(())
    }
}
