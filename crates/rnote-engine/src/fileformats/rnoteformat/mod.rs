//! Loading and saving Rnote's `.rnote` file format
//!
//! Older formats can be added, with the naming scheme `RnoteFileMaj<X>Min<Y>`,
//! where X: semver major, Y: semver minor version.
//!
//! Then [TryFrom] can be implemented to allow conversions and chaining from older to newer versions.

// Modules
pub(crate) mod legacy;
pub(crate) mod maj0min12;
pub(crate) mod methods;

// Imports
use super::{FileFormatLoader, FileFormatSaver};
use crate::engine::EngineSnapshot;
use legacy::LegacyRnoteFile;
use maj0min12::RnoteFileMaj0Min12;
use methods::{CompM, SerM};
use std::io::Write;

pub type RnoteFile = maj0min12::RnoteFileMaj0Min12;
pub type RnoteHeader = maj0min12::RnoteHeaderMaj0Min12;

pub const MAGIC_NUMBER: [u8; 9] = [0x52, 0x4e, 0x4f, 0x54, 0x45, 0xce, 0xa6, 0xce, 0x9b];

impl FileFormatSaver for RnoteFile {
    fn save_as_bytes(&self, _file_name: &str) -> anyhow::Result<Vec<u8>> {
        let json_header = serde_json::to_vec(&self.head)?;
        let header = [
            &MAGIC_NUMBER[..],
            &Self::VERSION[..],
            &u32::try_from(json_header.len())?.to_le_bytes(),
            &json_header,
        ]
        .concat();
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_all(&header)?;
        buffer.write_all(&self.body)?;
        Ok(buffer)
    }
}

impl FileFormatLoader for RnoteFile {
    fn load_from_bytes(bytes: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let magic_number = bytes
            .get(..9)
            .ok_or(anyhow::anyhow!("Failed to get magic number"))?;

        if magic_number != MAGIC_NUMBER {
            if magic_number[..2] == [0x1f, 0x8b] {
                return RnoteFile::try_from(LegacyRnoteFile::load_from_bytes(bytes)?);
            } else {
                Err(anyhow::anyhow!("Unkown file format"))?;
            }
        }

        let mut version: [u8; 3] = [0; 3];
        version.copy_from_slice(
            bytes
                .get(9..12)
                .ok_or(anyhow::anyhow!("Failed to get version"))?,
        );

        let mut header_size: [u8; 4] = [0; 4];
        header_size.copy_from_slice(
            bytes
                .get(12..16)
                .ok_or(anyhow::anyhow!("Failed to get header size"))?,
        );
        let header_size = u32::from_le_bytes(header_size);

        let header_slice = bytes
            .get(16..16 + usize::try_from(header_size)?)
            .ok_or(anyhow::anyhow!("File head missing"))?;

        let body_slice = bytes
            .get(16 + usize::try_from(header_size)?..)
            .ok_or(anyhow::anyhow!("File body missing"))?;

        Ok(Self {
            head: serde_json::from_slice(header_slice)?,
            body: body_slice.to_vec(),
        })
    }
}

impl TryFrom<RnoteFileMaj0Min12> for EngineSnapshot {
    type Error = anyhow::Error;

    fn try_from(value: RnoteFileMaj0Min12) -> Result<Self, Self::Error> {
        let uc_size = usize::try_from(value.head.uc_size).unwrap_or(usize::MAX);
        let uc_body = value.head.compression.decompress(uc_size, value.body)?;
        value.head.serialization.deserialize(&uc_body)
    }
}

impl TryFrom<&EngineSnapshot> for RnoteFile {
    type Error = anyhow::Error;

    fn try_from(value: &EngineSnapshot) -> Result<Self, Self::Error> {
        let serialization = SerM::Json;
        let compression = CompM::Zstd;
        let uc_data = serialization.serialize(value)?;
        let uc_size = uc_data.len() as u64;
        let data = compression.compress(uc_data)?;

        Ok(Self {
            head: RnoteHeader {
                compression,
                serialization,
                uc_size,
            },
            body: data,
        })
    }
}
