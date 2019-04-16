#[test]
fn read_file() -> std::io::Result<()> {
    let data = include_bytes!("test.pcap");
    let (head, reader) = example_pcap::read(&data[..])?;
    assert_eq!(head.magic, 0xa1b2_c3d4);
    let pkts = reader.collect::<Vec<example_pcap::Packet>>();
    assert_eq!(pkts.len(), 10);
    Ok(())
}
