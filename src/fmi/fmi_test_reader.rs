use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{
    path::{PathDistance, PathQuery},
    types::{Distance, VertexId},
};

pub fn read_tests(file: &std::path::Path) -> Option<Vec<PathDistance>> {
    let mut tests = Vec::new();
    let reader_test = BufReader::new(File::open(&file).unwrap());
    let mut test_lines = reader_test.lines().flatten();
    test_lines.next();
    while let Some(line) = test_lines.next() {
        let mut parts = line.split_whitespace();

        let source = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
        let target = VertexId::new(parts.next().unwrap().parse().ok().unwrap());
        let query = PathQuery::new(source, target);

        let distance: Option<Distance> =
            parts.next().unwrap().parse().ok().map(|x| Distance::new(x));

        let validation = PathDistance::new(query, distance);

        tests.push(validation);
    }

    Some(tests)
}
