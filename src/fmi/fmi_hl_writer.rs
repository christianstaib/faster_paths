use std::io::{self, Write};

use serde::Serialize;

use crate::fmi::fmi_hl_format::BINARY_HL_MAGIC;
use crate::hub_labeling::HubLabeling;
use crate::types::Distance;

pub fn write_fmi_hl<W: Write, D: Distance + Serialize>(
    mut out: W,
    hub_labeling: &HubLabeling<D>,
) -> io::Result<()> {
    out.write_all(BINARY_HL_MAGIC)?;
    bincode::serialize_into(out, hub_labeling)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}
