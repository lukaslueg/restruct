#[derive(restruct_derive::Struct)]
#[fmt = "<I2Hi3I"]
struct PcapHeader;

#[derive(restruct_derive::Struct)]
#[fmt = "<4I"]
struct PcapPacketHeader;

#[derive(Debug)]
pub struct Header {
    pub magic: u32,
    pub version_major: u16,
    pub version_minor: u16,
    pub ts_correction: i32,
    pub ts_accuracy: u32,
    pub snaplen: u32,
    pub datalink: u32,
}

impl Header {
    fn read_from<R: std::io::Read>(mut inp: R) -> std::io::Result<Self> {
        let (magic, version_major, version_minor, ts_correction, ts_accuracy, snaplen, datalink) =
            PcapHeader::read_from(&mut inp)?;

        if magic != 0xa1b2_c3d4 {
            panic!("oh noes, we don't support this!");
        }

        Ok(Self {
            magic,
            version_major,
            version_minor,
            ts_correction,
            ts_accuracy,
            snaplen,
            datalink,
        })
    }
}

#[derive(Debug)]
pub struct Packet {
    pub ts_sec: u32,
    pub ts_usec: u32,
    pub orig_len: u32,
    pub data: Vec<u8>,
}

impl Packet {
    fn read_from<R: std::io::Read>(mut inp: R) -> std::io::Result<Self> {
        let (ts_sec, ts_usec, incl_len, orig_len) = PcapPacketHeader::read_from(&mut inp)?;
        let mut data = vec![0; incl_len as usize];
        inp.read_exact(&mut data).map(|()| Self {
            ts_sec,
            ts_usec,
            orig_len,
            data,
        })
    }
}

pub fn read<R: std::io::Read>(
    mut inp: R,
) -> std::io::Result<(Header, impl Iterator<Item = Packet>)> {
    let head = Header::read_from(&mut inp)?;
    let reader = std::iter::from_fn(move || {
        Packet::read_from(&mut inp).ok() /* Yes, we cheat */
    });
    Ok((head, reader))
}
