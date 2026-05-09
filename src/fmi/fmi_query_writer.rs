use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::path::PathQuery;

pub fn write_queries(path: &PathBuf, queries: &[PathQuery]) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "{}", queries.len())?;
    for query in queries {
        writeln!(
            writer,
            "{} {}",
            query.source.as_usize(),
            query.target.as_usize()
        )?;
    }

    Ok(())
}
