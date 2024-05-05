use llhls_rs::read_playlist;
use std::fs;

#[test]
fn parse_ll_hls_basic() {
    let file = fs::File::open("tests/resources/ll-hls.m3u8").expect("Opened test file");
    assert!(read_playlist(file).is_ok())
}
