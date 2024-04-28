use fluent_uri::Uri;
use llhls_rs::PartialSegment;
use std::fs;

#[test]
fn parse_ll_hls_basic() {
    let playlist = fs::read_to_string("tests/resources/ll-hls.m3u8").unwrap();
    println!("playlist: {}", playlist);
    ()
}

#[test]
fn fmt_partial_segment() {
    let part = PartialSegment {
        part_duration: 0.33,
        uri: Uri::parse_from("part.mp4".to_owned()).unwrap(),
        independent: Option::None,
    };
    println!("part: {}", part);
}

#[test]
fn parse_partial_segment() {
    let part = "#EXT-X-PART:DURATION=0.33334,URI=\"filePart272.a.mp4\"";
    let _partial_segment: PartialSegment = part.parse().unwrap();
}
