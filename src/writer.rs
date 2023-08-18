use acid_io::{Read, Write};
use bitvec::prelude::*;

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

    pub fn write_bits(&mut self, bits: &BitSlice<u8, Msb0>) {
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
        bits.read_to_end(&mut buf).unwrap();
        self.bits_written = buf.len() * 8;
        self.leftover = bits.to_bitvec();
        self.inner.write_all(&buf).unwrap();
        #[cfg(feature = "logging")]
        log::trace!("wrote {} bits", buf.len() * 8);
    }

    #[inline]
    pub fn write_bytes(&mut self, buf: &[u8]) {
        #[cfg(feature = "logging")]
        log::trace!("writing {} bytes", buf.len());
        if !self.leftover.is_empty() {
            #[cfg(feature = "logging")]
            log::trace!("leftover exists");
            // TODO: we could check here and only send the required bits to finish the byte?
            // (instead of sending the entire thing)
            self.write_bits(&mut BitVec::from_slice(buf));
        } else {
            self.inner.write_all(buf).unwrap();
            self.bits_written = buf.len() * 8;
        }
    }

    #[inline]
    pub fn finalize(&mut self) {
        if !self.leftover.is_empty() {
            #[cfg(feature = "logging")]
            log::trace!("finalized: {} bits leftover", self.leftover.len());
            self.leftover.extend_from_bitslice(&bitvec![u8, Msb0; 0; 8 - self.leftover.len()]);
            let mut buf = vec![];
            self.leftover.read_to_end(&mut buf).unwrap();
            self.inner.write_all(&buf).unwrap();
            #[cfg(feature = "logging")]
            log::trace!("finalized: wrote {} bits", buf.len() * 8);
        }
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
