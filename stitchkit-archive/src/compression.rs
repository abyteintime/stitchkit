use std::io::{Read, Seek, SeekFrom};

use anyhow::{bail, ensure, Context};
use rust_lzo::{LZOContext, LZOError};
use stitchkit_core::binary::Deserializer;
use tracing::{debug, trace};

use crate::sections::{CompressedChunkBlock, CompressedChunkHeader, Summary};

impl Summary {
    const COMPRESSION_NONE: u32 = 0;
    const COMPRESSION_LZO: u32 = 2;

    pub fn decompress_archive_to_memory(
        &self,
        mut deserializer: Deserializer<impl Read + Seek>,
    ) -> anyhow::Result<Vec<u8>> {
        debug!("Loading entire file to memory");
        let size = deserializer.seek(SeekFrom::End(0))?;
        let mut buffer = vec![0; size as usize];
        deserializer.seek(SeekFrom::Start(0))?;
        deserializer
            .read_exact(&mut buffer)
            .context("cannot read entire archive to memory")?;
        debug!(
            "Size on disk: {} bytes | {:.2} MiB",
            buffer.len(),
            buffer.len() as f64 / 1024.0 / 1024.0
        );

        ensure!(
            self.compression_kind == Self::COMPRESSION_NONE
                || self.compression_kind == Self::COMPRESSION_LZO,
            "unsupported compression kind {}",
            self.compression_kind
        );

        if let Some(output_buffer_size) = self
            .compressed_chunks
            .iter()
            .map(|chunk| (chunk.uncompressed_offset + chunk.uncompressed_size) as usize)
            .max()
        {
            debug!(
                "Decompressing archive (uncompressed size: {output_buffer_size} bytes | {:.2} MiB)",
                output_buffer_size as f64 / 1024.0 / 1024.0
            );
            let mut decompressed_buffer = buffer.clone();
            decompressed_buffer.resize(output_buffer_size, 0);
            for (i, chunk) in self.compressed_chunks.iter().enumerate() {
                let compressed_offset = chunk.compressed_offset as usize;

                debug!("Decompressing compressed chunk {i} in archive (at offset {compressed_offset:08x})");
                let mut deserializer = Deserializer::from_buffer(buffer.as_slice());
                deserializer.seek(SeekFrom::Current(compressed_offset as i64))?;
                let header = deserializer.deserialize::<CompressedChunkHeader>()?;
                trace!("Read header: {header:#?}");

                let block_count =
                    (header.sum.uncompressed_size + header.block_size - 1) / header.block_size;
                let mut blocks = Vec::with_capacity(block_count as usize);
                for _ in 0..block_count {
                    blocks.push(deserializer.deserialize::<CompressedChunkBlock>()?);
                }
                trace!("Read {} blocks", blocks.len());

                let mut in_position = deserializer.position() as usize;
                let mut out_position = chunk.uncompressed_offset as usize;
                for (i, block) in blocks.iter().enumerate() {
                    trace!("Decompressing block {i} at {in_position:08x}: {block:#?}");
                    let (_, result) = LZOContext::decompress_to_slice(
                        &buffer[in_position..in_position + block.compressed_size as usize],
                        &mut decompressed_buffer
                            [out_position..out_position + block.uncompressed_size as usize],
                    );
                    in_position += block.compressed_size as usize;
                    out_position += block.uncompressed_size as usize;
                    if result != LZOError::OK {
                        let result = match result {
                            LZOError::OK => "OK",
                            LZOError::ERROR => "ERROR",
                            LZOError::OUT_OF_MEMORY => "OUT_OF_MEMORY",
                            LZOError::NOT_COMPRESSIBLE => "NOT_COMPRESSIBLE",
                            LZOError::INPUT_OVERRUN => "INPUT_OVERRUN",
                            LZOError::OUTPUT_OVERRUN => "OUTPUT_OVERRUN",
                            LZOError::LOOKBEHIND_OVERRUN => "LOOKBEHIND_OVERRUN",
                            LZOError::EOF_NOT_FOUND => "EOF_NOT_FOUND",
                            LZOError::INPUT_NOT_CONSUMED => "INPUT_NOT_CONSUMED",
                            LZOError::NOT_YET_IMPLEMENTED => "NOT_YET_IMPLEMENTED",
                            LZOError::INVALID_ARGUMENT => "INVALID_ARGUMENT",
                        };
                        bail!("failed to decompress block (LZO error {result})");
                    }
                }
            }
            Ok(decompressed_buffer)
        } else {
            debug!("Archive is not compressed; nothing to do");
            Ok(buffer)
        }
    }
}
