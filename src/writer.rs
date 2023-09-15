use bitvec::prelude::*;
use no_std_io::io::{Read, Write};

#[cfg(feature = "logging")]
use log;

use crate::{prelude::NeedSize, DekuError};

/// Container to use with `from_reader`
pub struct Writer<W: Write> {
    inner: W,
    pub leftover: BitVec<u8, Msb0>,
    pub bits_written: usize,
}

impl<W: Write> Writer<W> {
    #[inline]
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            leftover: BitVec::new(),
            bits_written: 0,
        }
    }

    #[inline]
    pub fn write_bits(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), DekuError> {
        #[cfg(feature = "logging")]
        log::trace!("writing {} bits", bits.len());
        let mut bits = if self.leftover.is_empty() {
            bits
        } else {
            #[cfg(feature = "logging")]
            log::trace!("pre-pending {} bits", self.leftover.len());
            self.leftover.extend_from_bitslice(bits);
            &mut self.leftover
        };

        // TODO: with_capacity?
        let mut buf = vec![];
        if let Err(_) = bits.read_to_end(&mut buf) {
            return Err(DekuError::WriteError);
        }
        self.bits_written = buf.len() * 8;
        self.leftover = bits.to_bitvec();
        if let Err(_) = self.inner.write_all(&buf) {
            return Err(DekuError::WriteError);
        }
        #[cfg(feature = "logging")]
        log::trace!("wrote {} bits", buf.len() * 8);

        Ok(())
    }

    #[inline]
    pub fn write_bytes(&mut self, buf: &[u8]) -> Result<(), DekuError> {
        #[cfg(feature = "logging")]
        log::trace!("writing {} bytes", buf.len());
        if !self.leftover.is_empty() {
            #[cfg(feature = "logging")]
            log::trace!("leftover exists");
            // TODO: we could check here and only send the required bits to finish the byte?
            // (instead of sending the entire thing)
            self.write_bits(&mut BitVec::from_slice(buf))?;
        } else {
            if let Err(_) = self.inner.write_all(buf) {
                return Err(DekuError::WriteError);
            }
            self.bits_written = buf.len() * 8;
        }

        Ok(())
    }

    #[inline]
    pub fn finalize(&mut self) -> Result<(), DekuError> {
        if !self.leftover.is_empty() {
            #[cfg(feature = "logging")]
            log::trace!("finalized: {} bits leftover", self.leftover.len());
            self.leftover
                .extend_from_bitslice(&bitvec![u8, Msb0; 0; 8 - self.leftover.len()]);
            let mut buf = vec![];
            if let Err(_) = self.leftover.read_to_end(&mut buf) {
                return Err(DekuError::WriteError);
            }
            if let Err(_) = self.inner.write_all(&buf) {
                return Err(DekuError::WriteError);
            }
            #[cfg(feature = "logging")]
            log::trace!("finalized: wrote {} bits", buf.len() * 8);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexlit::hex;

    #[test]
    fn test_writer() {
        let mut out_buf = vec![];
        let mut writer = Writer::new(&mut out_buf);

        let mut input = hex!("aa");
        writer.write_bytes(&mut input);

        let mut bv = BitVec::<u8, Msb0>::from_slice(&[0xbb]);
        writer.write_bits(&mut bv);

        let mut bv = bitvec![u8, Msb0; 1, 1, 1, 1];
        writer.write_bits(&mut bv);
        let mut bv = bitvec![u8, Msb0; 0, 0, 0, 1];
        writer.write_bits(&mut bv);

        let mut input = hex!("aa");
        writer.write_bytes(&mut input);

        let mut bv = bitvec![u8, Msb0; 0, 0, 0, 1];
        writer.write_bits(&mut bv);
        let mut bv = bitvec![u8, Msb0; 1, 1, 1, 1];
        writer.write_bits(&mut bv);

        let mut bv = bitvec![u8, Msb0; 0, 0, 0, 1];
        writer.write_bits(&mut bv);

        let mut input = hex!("aa");
        writer.write_bytes(&mut input);

        let mut bv = bitvec![u8, Msb0; 1, 1, 1, 1];
        writer.write_bits(&mut bv);

        assert_eq!(
            &mut out_buf,
            &mut vec![0xaa, 0xbb, 0xf1, 0xaa, 0x1f, 0x1a, 0xaf]
        );
    }
}
