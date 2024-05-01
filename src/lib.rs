use derive_builder::Builder;
use fluent_uri::Uri;
use std::{collections::HashMap, fmt, str::FromStr};

#[derive(Builder)]
struct MediaPlaylist {
    target_duration: u32,
    version: u32,
    part_target: u32,
    msn: u32,
    media_segments: Vec<MediaSegment>,
    skip: Option<Skip>,
    preload_hint: PreloadHint,
}

#[derive(Builder)]
struct ServerControl {
    can_block_reload: bool,
    part_hold_back: u32,
    can_skip_until: f32,
}

#[derive(Clone, Builder)]
struct MediaSegment {
    duration: f32,
    uri: Uri<String>,
    partial_segments: Vec<PartialSegment>,
}

#[derive(Clone, Builder)]
pub struct PartialSegment {
    pub part_duration: f32,
    pub uri: Uri<String>,
    pub independent: Option<bool>,
    // TODO: BYTERANGE and GAP
}

#[derive(Clone, Builder)]
pub struct Skip {
    pub skipped_segments: u32,
    pub recently_removed_dateranges: Vec<String>,
}

#[derive(Clone, Builder)]
pub struct PreloadHint {
    pub r#type: PreloadHintType,
    pub uri: Uri<String>,
    pub byterange_start: Option<u32>,
    pub byterange_length: Option<u32>,
}

#[derive(Clone)]
pub enum PreloadHintType {
    Part,
    Map,
}

pub enum SkipAttribute {
    SkippedSegments,
    RecentlyRemovedDateRanges,
}

impl fmt::Display for PartialSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut attrs = vec![
            ("DURATION", self.part_duration.to_string()),
            ("URI", self.uri.to_string()),
        ];
        if let Some(independent) = self.independent {
            attrs.push((
                "INDEPENDENT",
                if independent {
                    "YES".to_string()
                } else {
                    "FALSE".to_string()
                },
            ));
        }
        let attrs_str: Vec<String> = attrs
            .into_iter()
            .map(|(name, value)| format!("{}={}", name, value))
            .collect();
        write!(f, "#EXT-X-PART:{}", attrs_str.join(","))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTagError;

impl FromStr for PartialSegment {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let attrs = s.strip_prefix("#EXT-X-PART:").ok_or(ParseTagError)?;
        let res: HashMap<String, String> = attrs
            .split(",")
            .filter_map(|x| {
                x.split_once('=')
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect();
        Ok(PartialSegment {
            part_duration: res
                .get("DURATION")
                .ok_or(ParseTagError)?
                .parse()
                .map_err(|_| ParseTagError)?,
            uri: Uri::parse_from(
                res.get("URI")
                    .ok_or(ParseTagError)?
                    .trim_matches('"')
                    .to_owned(),
            )
            .map_err(|_| ParseTagError)?,
            independent: res
                .get("INDEPENDENT")
                .map(|s| s.parse().map_err(|_| ParseTagError))
                .transpose()?,
        })
    }
}
