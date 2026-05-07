use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::{path::PathDistance, types::Distance};

pub fn write_tests<D: Distance + Display>(
    path: &PathBuf,
    tests: &[PathDistance<D>],
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "{}", tests.len())?;
    for test in tests {
        let query = test.query();
        let distance = test
            .distance()
            .map(|distance| distance.to_string())
            .unwrap_or_else(|| "None".to_string());

        writeln!(
            writer,
            "{} {} {}",
            query.source.as_usize(),
            query.target.as_usize(),
            distance
        )?;
    }

    Ok(())
}
