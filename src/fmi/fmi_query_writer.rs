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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use crate::types::VertexId;

    #[test]
    fn writes_queries() {
        let path = std::env::temp_dir().join("ch_fmi_queries.txt");
        let queries = vec![
            PathQuery {
                source: VertexId::new(1),
                target: VertexId::new(2),
            },
            PathQuery {
                source: VertexId::new(3),
                target: VertexId::new(4),
            },
        ];

        write_queries(&path, &queries).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "2\n1 2\n3 4\n");
        fs::remove_file(path).unwrap();
    }
}
