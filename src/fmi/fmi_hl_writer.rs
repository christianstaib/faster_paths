use std::io::{self, Write};

use serde::Serialize;

use crate::hub_labeling::HubLabeling;
use crate::types::Distance;

pub fn write_fmi_hl<W: Write, D: Distance + Serialize>(
    out: W,
    hub_labeling: &HubLabeling<D>,
) -> io::Result<()> {
    postcard::to_io(hub_labeling, out)
        .map(|_| ())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
