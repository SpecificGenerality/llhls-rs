use chrono::{DateTime, Utc};
use derive_builder::Builder;
use fluent_uri::Uri;
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

#[derive(Builder)]
pub struct MediaPlaylist {
    target_duration: u32,
    version: u32,
    part_inf: PartInf,
    media_sequence_number: u32,
    media_segments: Vec<MediaSegment>,
    skip: Option<Skip>,
    preload_hint: Option<PreloadHint>,
    rendition_reports: Vec<RenditionReport>,
    server_control: ServerControl,
}

#[derive(Builder, Clone)]
struct PartInf {
    part_target: f32,
}

#[derive(Builder, Clone)]
struct ServerControl {
    can_block_reload: bool,
    part_hold_back: f32,
    can_skip_until: f32,
}

enum YesNo {
    Yes,
    No,
}

impl FromStr for YesNo {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "YES" => Ok(YesNo::Yes),
            "NO" => Ok(YesNo::No),
            _ => Err(ParseAttributeError),
        }
    }
}

impl From<YesNo> for bool {
    fn from(value: YesNo) -> Self {
        match value {
            YesNo::Yes => true,
            YesNo::No => false,
        }
    }
}

impl FromStr for ServerControl {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = ServerControlBuilder::default();
        read_attributes::<ServerControlAttribute, ServerControlBuilder>(s, &mut builder)
            .map_err(|_| ParseTagError)?;
        builder.build().map_err(|_| ParseTagError)
    }
}

#[derive(Clone, Builder, Default)]
struct MediaSegment {
    duration: f32,
    uri: Uri<String>,
    partial_segments: Vec<PartialSegment>,
    program_date_time: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, Builder)]
pub struct PartialSegment {
    pub part_duration: f32,
    pub uri: String,
    pub independent: Option<bool>,
    // TODO: BYTERANGE and GAP
}

impl FromStr for PartialSegment {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = PartialSegmentBuilder::default();
        read_attributes::<PartialSegmentAttribute, PartialSegmentBuilder>(s, &mut builder)
            .map_err(|_| ParseTagError)?;
        if builder.independent.is_none() {
            builder.independent(None);
        }
        builder.build().map_err(|_| ParseTagError)
    }
}

#[derive(Clone, Builder)]
pub struct Skip {
    pub skipped_segments: u32,
    pub recently_removed_dateranges: Vec<String>,
}

#[derive(Clone, Builder)]
pub struct PreloadHint {
    pub r#type: PreloadHintType,
    pub uri: String,
    pub byterange_start: Option<u32>,
    pub byterange_length: Option<u32>,
}

#[derive(Clone)]
pub enum PreloadHintType {
    Part,
    Map,
}

pub enum MediaPlaylistTag {
    TargetDuration,
    Version,
    PartInf,
    MediaSequence,
    Skip,
    PreloadHint,
    RenditionReport,
    ServerControl,
}

impl FromStr for MediaPlaylistTag {
    type Err = ParseTagError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "EXT-X-TARGETDURATION" => Ok(MediaPlaylistTag::TargetDuration),
            "EXT-X-VERSION" => Ok(MediaPlaylistTag::Version),
            "EXT-X-PART-INF" => Ok(MediaPlaylistTag::PartInf),
            "EXT-X-MEDIA-SEQUENCE" => Ok(MediaPlaylistTag::MediaSequence),
            "EXT-X-SKIP" => Ok(MediaPlaylistTag::Skip),
            "EXT-X-PRELOAD-HINT" => Ok(MediaPlaylistTag::PreloadHint),
            "EXT-X-RENDITION-REPORT" => Ok(MediaPlaylistTag::RenditionReport),
            "EXT-X-SERVER-CONTROL" => Ok(MediaPlaylistTag::ServerControl),
            _ => Err(ParseTagError),
        }
    }
}

pub enum ServerControlAttribute {
    CanBlockReload,
    PartHoldBack,
    CanSkipUntil,
}

impl FromStr for ServerControlAttribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CAN-BLOCK-RELOAD" => Ok(ServerControlAttribute::CanBlockReload),
            "PART-HOLD-BACK" => Ok(ServerControlAttribute::PartHoldBack),
            "CAN-SKIP-UNTIL" => Ok(Self::CanSkipUntil),
            _ => Err(ParseAttributeError),
        }
    }
}

trait Attribute<B> {
    fn read(&self, builder: &mut B, attribute: &str) -> Result<(), ParseAttributeError>;
}

impl Attribute<ServerControlBuilder> for ServerControlAttribute {
    fn read(
        &self,
        builder: &mut ServerControlBuilder,
        attribute: &str,
    ) -> Result<(), ParseAttributeError> {
        match self {
            ServerControlAttribute::CanBlockReload => {
                builder.can_block_reload(
                    YesNo::from_str(attribute)
                        .map_err(|_| ParseAttributeError)?
                        .into(),
                );
            }
            ServerControlAttribute::PartHoldBack => {
                builder.part_hold_back(f32::from_str(attribute).map_err(|_| ParseAttributeError)?);
            }
            ServerControlAttribute::CanSkipUntil => {
                builder.can_skip_until(f32::from_str(attribute).map_err(|_| ParseAttributeError)?);
            }
            _ => todo!(),
        }
        Ok(())
    }
}

pub enum PartialSegmentAttribute {
    Duration,
    Uri,
    Independent,
}

impl FromStr for PartialSegmentAttribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DURATION" => Ok(PartialSegmentAttribute::Duration),
            "URI" => Ok(PartialSegmentAttribute::Uri),
            "INDEPENDENT" => Ok(PartialSegmentAttribute::Independent),
            _ => Err(ParseAttributeError),
        }
    }
}

impl Attribute<PartialSegmentBuilder> for PartialSegmentAttribute {
    fn read(
        &self,
        builder: &mut PartialSegmentBuilder,
        attribute: &str,
    ) -> Result<(), ParseAttributeError> {
        match self {
            PartialSegmentAttribute::Duration => {
                builder.part_duration(f32::from_str(attribute).map_err(|_| ParseAttributeError)?);
            }
            PartialSegmentAttribute::Uri => {
                builder.uri(attribute.to_string());
            }
            PartialSegmentAttribute::Independent => {
                builder.independent(Some(
                    YesNo::from_str(attribute)
                        .map_err(|_| ParseAttributeError)?
                        .into(),
                ));
            }
        }
        Ok(())
    }
}

pub enum MediaSegmentTag {
    Inf,
    Part,
    // Not strictly a tag, just makes things work nicer internally
    Uri,
    ProgramDateTime,
}

impl FromStr for MediaSegmentTag {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EXTINF" => Ok(MediaSegmentTag::Inf),
            "EXT-X-PART" => Ok(MediaSegmentTag::Part),
            "EXT-X-PROGRAM-DATE-TIME" => Ok(MediaSegmentTag::ProgramDateTime),
            // lol
            _ => Ok(MediaSegmentTag::Uri),
        }
    }
}

#[derive(Builder)]
pub struct Inf {
    duration: f32,
    uri: Uri<String>,
}

pub enum InfAttribute {
    Duration,
    Uri,
}

impl Attribute<InfBuilder> for InfAttribute {
    fn read(&self, builder: &mut InfBuilder, attribute: &str) -> Result<(), ParseAttributeError> {
        match self {
            InfAttribute::Duration => {
                builder.duration(f32::from_str(attribute).map_err(|_| ParseAttributeError)?)
            }
            InfAttribute::Uri => builder
                .uri(Uri::parse_from(attribute.to_string()).map_err(|_| ParseAttributeError)?),
        };
        Ok(())
    }
}

struct WrappedMediaSegmentBuilder {
    segment: MediaSegmentBuilder,
    parts: Vec<PartialSegment>,
}

impl Tag<WrappedMediaSegmentBuilder> for MediaSegmentTag {
    fn read(
        &self,
        builder: &mut WrappedMediaSegmentBuilder,
        attributes: &str,
    ) -> Result<(), ParseTagError> {
        match self {
            MediaSegmentTag::Inf => {
                builder
                    .segment
                    // TODO: Clean up
                    .duration(
                        f32::from_str(attributes.split_once(',').ok_or(ParseTagError)?.0)
                            .map_err(|_| ParseTagError)?,
                    );
                Ok(())
            }
            MediaSegmentTag::Part => {
                builder
                    .parts
                    .push(PartialSegment::from_str(attributes).map_err(|_| ParseTagError)?);
                Ok(())
            }
            MediaSegmentTag::Uri => {
                builder
                    .segment
                    .uri(Uri::parse_from(attributes.to_string()).map_err(|_| ParseTagError)?);
                Ok(())
            }
            MediaSegmentTag::ProgramDateTime => {
                builder.segment.program_date_time(Some(
                    DateTime::from_str(attributes).map_err(|_| ParseTagError)?,
                ));
                Ok(())
            }
        }
    }
}

pub enum MediaSegmentAttribute {
    Duration,
    Uri,
}

impl Attribute<MediaSegmentBuilder> for MediaSegmentAttribute {
    fn read(
        &self,
        builder: &mut MediaSegmentBuilder,
        attribute: &str,
    ) -> Result<(), ParseAttributeError> {
        match self {
            MediaSegmentAttribute::Duration => {
                builder.duration(f32::from_str(attribute).map_err(|_| ParseAttributeError)?);
            }
            MediaSegmentAttribute::Uri => {
                builder
                    .uri(Uri::parse_from(attribute.to_string()).map_err(|_| ParseAttributeError)?);
            }
        }
        Ok(())
    }
}

#[derive(Builder, Clone)]
pub struct RenditionReport {
    uri: String,
    last_msn: u32,
    last_part: u32,
}
pub enum RenditionReportAttribute {
    Uri,
    LastMsn,
    LastPart,
}

impl Attribute<RenditionReportBuilder> for RenditionReportAttribute {
    fn read(
        &self,
        builder: &mut RenditionReportBuilder,
        attribute: &str,
    ) -> Result<(), ParseAttributeError> {
        match self {
            RenditionReportAttribute::Uri => {
                builder.uri(attribute.to_string());
            }
            RenditionReportAttribute::LastMsn => {
                builder.last_msn(u32::from_str(attribute).map_err(|_| ParseAttributeError)?);
            }
            RenditionReportAttribute::LastPart => {
                builder.last_part(u32::from_str(attribute).map_err(|_| ParseAttributeError)?);
            }
        }
        Ok(())
    }
}

pub enum PreloadHintAttribute {
    Type,
    Uri,
}

impl FromStr for RenditionReport {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder: RenditionReportBuilder = RenditionReportBuilder::default();
        read_attributes::<RenditionReportAttribute, RenditionReportBuilder>(s, &mut builder)
            .map_err(|_| ParseTagError)?;
        builder.build().map_err(|_| ParseTagError)
    }
}

impl FromStr for RenditionReportAttribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "URI" => Ok(RenditionReportAttribute::Uri),
            "LAST-MSN" => Ok(RenditionReportAttribute::LastMsn),
            "LAST-PART" => Ok(RenditionReportAttribute::LastPart),
            _ => Err(ParseAttributeError),
        }
    }
}

impl FromStr for PreloadHintType {
    type Err = ParseAttributeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "PART" => Ok(PreloadHintType::Part),
            "MAP" => Ok(PreloadHintType::Map),
            _ => Err(ParseAttributeError),
        }
    }
}

impl Attribute<PreloadHintBuilder> for PreloadHintAttribute {
    fn read(
        &self,
        builder: &mut PreloadHintBuilder,
        attribute: &str,
    ) -> Result<(), ParseAttributeError> {
        match self {
            PreloadHintAttribute::Type => {
                builder.r#type(PreloadHintType::from_str(attribute)?);
            }
            PreloadHintAttribute::Uri => {
                builder.uri(attribute.to_string());
            }
        }
        Ok(())
    }
}

trait Tag<B> {
    fn read(&self, builder: &mut B, attributes: &str) -> Result<(), ParseTagError>;
}

#[derive(Builder)]
struct WrappedMediaPlaylistBuilder {
    playlist: MediaPlaylistBuilder,
    rendition_reports: Vec<RenditionReport>,
    media_segments: Vec<MediaSegment>,
}

impl FromStr for PreloadHintAttribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TYPE" => Ok(PreloadHintAttribute::Type),
            "URI" => Ok(PreloadHintAttribute::Uri),
            _ => Err(ParseAttributeError),
        }
    }
}

impl FromStr for PreloadHint {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = PreloadHintBuilder::default();
        read_attributes::<PreloadHintAttribute, PreloadHintBuilder>(s, &mut builder)
            .map_err(|_| ParseTagError)?;
        if builder.byterange_start.is_none() {
            builder.byterange_start(None);
        }
        if builder.byterange_length.is_none() {
            builder.byterange_length(None);
        }
        builder.build().map_err(|_| ParseTagError)
    }
}

impl Tag<WrappedMediaPlaylistBuilder> for MediaPlaylistTag {
    fn read(
        &self,
        builder: &mut WrappedMediaPlaylistBuilder,
        attributes: &str,
    ) -> Result<(), ParseTagError> {
        match self {
            MediaPlaylistTag::TargetDuration => {
                builder
                    .playlist
                    .target_duration(u32::from_str(attributes).map_err(|_| ParseTagError)?);
                Ok(())
            }
            MediaPlaylistTag::Version => {
                builder
                    .playlist
                    .version(u32::from_str(attributes).map_err(|_| ParseTagError)?);
                Ok(())
            }
            MediaPlaylistTag::PartInf => {
                builder
                    .playlist
                    .part_inf(PartInf::from_str(attributes).map_err(|_| ParseTagError)?);
                Ok(())
            }
            MediaPlaylistTag::MediaSequence => {
                builder
                    .playlist
                    .media_sequence_number(u32::from_str(attributes).map_err(|_| ParseTagError)?);
                Ok(())
            }
            MediaPlaylistTag::Skip => {
                builder
                    .playlist
                    .skip(Some(Skip::from_str(attributes).map_err(|_| ParseTagError)?));
                Ok(())
            }
            MediaPlaylistTag::PreloadHint => {
                builder.playlist.preload_hint(Some(
                    PreloadHint::from_str(attributes).map_err(|_| ParseTagError)?,
                ));
                Ok(())
            }
            MediaPlaylistTag::RenditionReport => {
                builder
                    .rendition_reports
                    .push(RenditionReport::from_str(attributes).map_err(|_| ParseTagError)?);
                Ok(())
            }
            MediaPlaylistTag::ServerControl => {
                builder.playlist.server_control(
                    ServerControl::from_str(attributes).map_err(|_| ParseTagError)?,
                );
                Ok(())
            }
        }
    }
}

pub enum PartInfAttribute {
    PartTarget,
}

impl Attribute<PartInfBuilder> for PartInfAttribute {
    fn read(
        &self,
        builder: &mut PartInfBuilder,
        attribute: &str,
    ) -> Result<(), ParseAttributeError> {
        match self {
            PartInfAttribute::PartTarget => {
                builder.part_target(f32::from_str(attribute).map_err(|_| ParseAttributeError)?);
                Ok(())
            }
        }
    }
}

impl FromStr for PartInfAttribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PART-TARGET" => Ok(PartInfAttribute::PartTarget),
            _ => Err(ParseAttributeError),
        }
    }
}

impl FromStr for PartInf {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder: PartInfBuilder = PartInfBuilder::default();
        read_attributes::<PartInfAttribute, PartInfBuilder>(s, &mut builder)
            .map_err(|_| ParseTagError)?;
        builder.build().map_err(|_| ParseTagError)
    }
}

fn read_attributes<T, B>(s: &str, builder: &mut B) -> Result<(), ParseAttributeError>
where
    T: FromStr + Attribute<B>,
{
    let attributes: HashMap<String, String> = s
        .split(",")
        .filter_map(|x| {
            x.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect();
    for (k, v) in attributes {
        let attribute = T::from_str(&k).map_err(|_| ParseAttributeError)?;
        attribute.read(builder, &v)?;
    }
    Ok(())
}

pub enum SkipAttribute {
    SkippedSegments,
    RecentlyRemovedDateRanges,
}

impl FromStr for SkipAttribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SKIPPED-SEGMENTS" => Ok(SkipAttribute::SkippedSegments),
            "RECENTLY-REMOVED-DATERANGES" => Ok(SkipAttribute::RecentlyRemovedDateRanges),
            _ => Err(ParseAttributeError),
        }
    }
}

impl Attribute<SkipBuilder> for SkipAttribute {
    fn read(&self, builder: &mut SkipBuilder, attribute: &str) -> Result<(), ParseAttributeError> {
        match self {
            SkipAttribute::SkippedSegments => {
                builder
                    .skipped_segments(u32::from_str(attribute).map_err(|_| ParseAttributeError)?);
                Ok(())
            }
            SkipAttribute::RecentlyRemovedDateRanges => {
                builder.recently_removed_dateranges(
                    attribute.split('\t').map(|s| s.to_string()).collect(),
                );
                Ok(())
            }
        }
    }
}

impl FromStr for Skip {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = SkipBuilder::default();
        read_attributes::<SkipAttribute, SkipBuilder>(s, &mut builder)
            .map_err(|_| ParseTagError)?;
        if builder.recently_removed_dateranges.is_none() {
            builder.recently_removed_dateranges(Vec::new());
        }
        builder.build().map_err(|_| ParseTagError)
    }
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

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAttributeError;

#[derive(Debug)]
pub enum ParsePlaylistError {
    EXT3U_TAG_MISSING,
    BUILDER_ERROR,
    IO_ERROR,
    UNRECOGNIZED_TAG { tag: String },
}

pub fn read_playlist(file: File) -> Result<MediaPlaylist, ParsePlaylistError> {
    let mut parser = BufReader::new(file);
    let mut line = String::new();
    parser
        .read_line(&mut line)
        .map_err(|_| ParsePlaylistError::IO_ERROR)?;
    if !line.trim().eq("#EXTM3U") {
        return Err(ParsePlaylistError::EXT3U_TAG_MISSING);
    }
    let mut builder = WrappedMediaPlaylistBuilder {
        playlist: MediaPlaylistBuilder::default(),
        rendition_reports: Vec::new(),
        media_segments: Vec::new(),
    };
    // Set some defaults so we don't forget later
    builder.playlist.skip(None);
    builder.playlist.preload_hint(None);
    let mut media_segment_builder = WrappedMediaSegmentBuilder {
        segment: MediaSegmentBuilder::default(),
        parts: Vec::new(),
    };
    line.clear();
    while let Ok(read_bytes) = parser.read_line(&mut line) {
        let is_uri = !line.starts_with('#') && !line.trim().is_empty();
        if line.starts_with("#EXT-X") || line.starts_with("#EXT") {
            let tag = line
                .trim_end()
                .split_once(':')
                .ok_or(ParseTagError)
                .map_err(|_| ParsePlaylistError::IO_ERROR)?;
            let tag_id = tag.0.split_once('#').ok_or(ParsePlaylistError::IO_ERROR)?.1;
            if let Ok(media_playlist_tag) = MediaPlaylistTag::from_str(tag_id) {
                media_playlist_tag
                    .read(&mut builder, tag.1)
                    .map_err(|_| ParsePlaylistError::BUILDER_ERROR)?;
            } else if let Ok(media_segment_tag) = MediaSegmentTag::from_str(tag_id) {
                media_segment_tag
                    .read(&mut media_segment_builder, tag.1)
                    .map_err(|_| ParsePlaylistError::BUILDER_ERROR)?;
            }
        } else if is_uri {
            if let Ok(media_segment_tag) = MediaSegmentTag::from_str(&line) {
                media_segment_tag
                    .read(&mut media_segment_builder, &line.trim_end())
                    .map_err(|_| ParsePlaylistError::BUILDER_ERROR)?;
            }
        }
        if is_uri || line.eq("EXT-X-ENDLIST") {
            if media_segment_builder.segment.program_date_time.is_none() {
                media_segment_builder.segment.program_date_time(None);
            }
            builder.media_segments.push(
                media_segment_builder
                    .segment
                    .partial_segments(media_segment_builder.parts)
                    .build()
                    .map_err(|_| ParsePlaylistError::BUILDER_ERROR)?,
            );
            media_segment_builder = WrappedMediaSegmentBuilder {
                segment: MediaSegmentBuilder::default(),
                parts: Vec::new(),
            };
        }
        if read_bytes == 0 {
            break;
        }
        line.clear();
    }
    builder
        .playlist
        .media_segments(builder.media_segments)
        .rendition_reports(builder.rendition_reports)
        .build()
        .map_err(|_| ParsePlaylistError::BUILDER_ERROR)
}
