use fluent_uri::Uri;
use llhls_rs::{read_playlist, PartialSegment};
use std::fs;

#[test]
fn parse_ll_hls_basic() {
    read_playlist(fs::File::open("tests/resources/ll-hls.m3u8").unwrap()).unwrap();
    ()
}

#[test]
fn parse_uri() {
    Uri::parse_from("fileSequence270.mp4".to_string()).unwrap();
    ()
}

#[test]
fn fmt_partial_segment() {
    let part = PartialSegment {
        part_duration: 0.33,
        uri: "\"part.mp4\"".to_string(),
        independent: Option::None,
    };
    println!("part: {}", part);
}

#[test]
fn parse_partial_segment() {
    let part = "#EXT-X-PART:DURATION=0.33334,URI=\"filePart272.a.mp4\"";
    let _partial_segment: PartialSegment = part.parse().unwrap();
}
