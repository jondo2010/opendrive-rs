extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate log;
extern crate serde_xml_rs;

mod util;
use util::flexible_boolean;

fn init_logger() {
    use log::{LogLevel, LogMetadata, LogRecord};

    struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &LogMetadata) -> bool {
            metadata.level() <= LogLevel::Debug
        }

        fn log(&self, record: &LogRecord) {
            if self.enabled(record.metadata()) {
                println!("{} - {}", record.level(), record.args());
            }
        }
    }

    let _ = log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Debug);
        Box::new(SimpleLogger)
    });
}

/*
fn string_as_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    deserializer.deserialize_f64(F64Visitor)
}

struct F64Visitor;
impl<'de> serde::de::Visitor<'de> for F64Visitor {
    type Value = f64;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representation of a f64")
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        s.parse::<f64>().map_err(serde::de::Error::custom)
    }
}
*/

mod odr_dateformat {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};
    const DATE_FORMAT: &'static str = "%a %b %d %H:%M:%S %Y";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(DATE_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, DATE_FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "OpenDRIVE")]
pub struct Root {
    pub header: Header,
    #[serde(default, rename = "road")]
    pub roads: Vec<Road>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "header", rename_all = "camelCase")]
pub struct Header {
    pub rev_major: u8,
    pub rev_minor: u8,
    pub name: String,
    pub version: String,
    #[serde(with = "odr_dateformat", default = "Header::default_date")]
    pub date: chrono::DateTime<chrono::Utc>,
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
    pub max_road: u32,
    pub max_junc: u32,
    pub max_prg: u32,
}
impl Header {
    fn default_date() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

/// 5.3.1 Road Header Record
/// The road header record defines the basic parameters of an individual road.
/// It is followed immediately by other records defining geometry and logical
/// properties of the road.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "road", rename_all = "camelCase")]
pub struct Road {
    /// name of the road
    pub name: String,
    /// total length of the reference line in the xy-plane
    pub length: f64,
    /// unique ID within database
    pub id: u8,
    /// ID of the junction to which the road belongs as a connecting road (= -1 for none)
    pub junction: i8,
    /// Road link record
    pub link: Option<Link>,
    pub plan_view: PlanView,
    pub elevation_profile: Option<ElevationProfile>,
    pub lateral_profile: Option<LateralProfile>,

    /// The lanes record contains a series of lane section records which define
    /// the characteristics of the road cross sections with respect to the lanes
    /// along the reference line.
    #[serde(default)]
    pub lanes: Option<Lanes>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "link", rename_all = "camelCase")]
pub struct Link {
    //#[serde(flatten)]
    pub predecessor: Option<LinkElement>,
    //#[serde(flatten)]
    pub successor: Option<LinkElement>,
    #[serde(default, rename = "neighbor")]
    pub neighbors: Vec<Neighbor>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LinkElement {
    pub element_type: ElementType,
    pub element_id: u32,
    pub contact_point: ContactPoint,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "neighbor", rename_all = "camelCase")]
pub struct Neighbor {
    pub side: Side,
    pub element_id: u32,
    pub direction: Direction,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ElementType {
    Road,
    Junction,
}
impl Default for ElementType {
    fn default() -> ElementType {
        ElementType::Road
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ContactPoint {
    Start,
    End,
}
impl Default for ContactPoint {
    fn default() -> ContactPoint {
        ContactPoint::Start
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Left,
    Right,
}
impl Default for Side {
    fn default() -> Side {
        Side::Left
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Same,
    Opposite,
}
impl Default for Direction {
    fn default() -> Direction {
        Direction::Same
    }
}

/// The plan view record contains a series of geometry records which define the
/// layout of the road's reference line in the x/y-plane (plan view).
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    #[serde(default, rename = "geometry")]
    pub geometries: Vec<Geometry>,
}

/// 5.3.4.1 Road Geometry Header Record
///
/// A sequence of road geometry records defines the layout of the road's
/// reference line in the in the x/yplane (plan view). The geometry records must
/// occur in ascending order (i.e. increasing s-position).
///
/// The geometry information is split into a header which is common to all
/// geometric elements and a subsequent bead containing the actual geometric
/// element’s data (depending on the type of geometric element).
///
/// Currently, four types of geometric elements are supported:
/// - straight lines
/// - spirals
/// - arcs
/// - cubic polynomials
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Geometry {
    /// m [0,∞[ start position (s-coordinate)
    pub s: f64,
    /// m ]-∞,∞[ start position (x inertial)
    pub x: f64,
    /// m ]-∞,∞[ start position (y inertial)
    pub y: f64,
    /// rad ]-∞,∞[ start orientation (inertial heading)
    pub hdg: f64,
    /// m [0,∞[ length of the element's reference line
    pub length: f64,

    #[serde(rename = "$value")]
    pub element: GeometryElement,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GeometryElement {
    /// This record describes a straight line as part of the road’s reference line
    Line,
    /// This record describes a spiral as part of the road’s reference line. For this type of spiral, the curvature change between start and end of the element is linear.
    #[serde(rename_all = "camelCase")]
    Spiral {
        /// double 1/m ]-∞,∞[ curvature at the start of the element
        curv_start: f64,
        /// double 1/m ]-∞,∞[ curvature at the end of the element
        curv_end: f64,
    },
    /// This record describes an arc as part of the road’s reference line.
    Arc {
        /// constant curvature throughout the element
        curvature: f64,
    },
    /// This record describes a cubic polynomial as part of the road’s reference line.
    Poly3 { a: f64, b: f64, c: f64, d: f64 },
}

/// 5.3.5 Road Elevation Profile Record
///
/// The elevation profile record contains a series of elevation records which
/// define the characteristics of the road's elevation along the reference line.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ElevationProfile {}

/// 5.3.6 Road Lateral Profile Record
///
/// The lateral profile record contains a series of superelevation and crossfall
/// records which define the characteristics of the road surface's banking along
/// the reference line.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LateralProfile {}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "lanes", rename_all = "camelCase")]
pub struct Lanes {
    #[serde(default, rename = "laneOffset")]
    pub lane_offsets: Vec<LaneOffset>,
    #[serde(default, rename = "laneSection")]
    pub lane_sections: Vec<LaneSection>,
}

/// 5.3.7.1 Road Lane Offset Record
///
/// The lane offset record defines a lateral shift of the lane reference line
/// (which is usually identical to the road reference line). This may be used
/// for an easy implementation of a (local) lateral shift of the lanes relative
/// to the road’s reference line. Especially the modeling of inner-city layouts
/// or "2+1" crosscountry road layouts can be facilitated considerably by this
/// feature.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneOffset {
    /// double m [0,∞[ start position (s-coordinate)
    pub s: f64,
    /// m ]-∞,∞[ parameter A, offset at s=0
    pub a: f64,
    /// 1 ]-∞,∞[ parameter B
    pub b: f64,
    /// 1/m ]-∞,∞[ parameter C
    pub c: f64,
    /// 1/m² ]-∞,∞[ parameter D
    pub d: f64,
}

/// 5.3.7.2 Road Lane Section Record
///
/// The lane section record defines the characteristics of a road cross-section.
/// It specifies the numbers and types of lanes and further features of the
/// lanes. At least one record must be defined in order to use a road. A lane
/// section record is valid until a new lane section record is defined. If
/// multiple lane section records are defined, they must be listed in ascending
/// order.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneSection {
    /// m - start position (s-coordinate)
    pub s: f64,
    /// lane section entry is valid for one side only (left or right, depending
    /// on the child entries)
    #[serde(default)]
    pub single_side: bool,

    pub left: Option<LaneSubSection>,
    pub center: LaneSubSection,
    pub right: Option<LaneSubSection>,
}

/// 5.3.7.2.1 Left / Center / Right Records
///
/// For easier navigation through a road description, the lanes under a lane
/// section are grouped into left, center and right lanes. At least one entry
/// (left, center or right) must be present.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneSubSection {
    pub lane: Vec<Lane>,
}

/// 5.3.7.2.1.1 Lane Record
///
/// Lane records are found within the left/center/right records. They define the
/// IDs of the actual lanes (and, therefore, their position on the road, see
/// conventions in 3). In order to prevent confusion, lane records should
/// represent the lanes from left to right (i.e. with descending ID). All
/// properties of the lanes are defined as children of the lane records.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Lane {
    /// - ]-∞,∞[ id of the lane (according to convention)
    pub id: i32,
    /// type of the lane
    #[serde(rename = "type")]
    pub lane_type: LaneType,
    /// "true" = keep lane on level, .i.e. do not apply superelevation or
    /// crossfall. "false" = apply superelevation and crossfall to this lane
    /// (default, also used if argument level is missing)
    #[serde(with = "flexible_boolean")]
    pub level: bool,

    /// In order to facilitate navigation through a road network on a per-lane
    /// basis, lanes should be provided with predecessor/successor information.
    /// Only when lanes end at a junction or have no physical link, this record
    /// should be omitted.
    pub link: Option<LaneLink>,

    #[serde(default, rename = "width")]
    pub widths: Vec<Width>,

    #[serde(default, rename = "roadMark")]
    pub road_marks: Vec<RoadMark>,
}
impl Default for Lane {
    fn default() -> Lane {
        Lane {
            level: false,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LaneType {
    None,
    Driving,
    Stop,
    Shoulder,
    Biking,
    Sidewalk,
    Border,
    Restricted,
    Parking,
    /// (full name: continuous two-way left turn lane)
    Bidirectional,
    Median,
    Special1,
    Special2,
    Special3,
    RoadWorks,
    Tram,
    Rail,
    Entry,
    Exit,
    OffRamp,
    OnRamp,
}
impl Default for LaneType {
    fn default() -> LaneType {
        LaneType::None
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "link", rename_all = "camelCase")]
pub struct LaneLink {
    pub predecessor: Option<LaneLinkElement>,
    pub successor: Option<LaneLinkElement>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneLinkElement {
    pub id: i32,
}

/// 5.3.7.2.1.1.2 Lane Width Record
///
/// Each lane within a road’s cross section can be provided with several width
/// entries. At least one entry must be defined for each lane, except for the
/// center lane which is, per convention, of zero width (see 3.2). Each entry
/// is valid until a new entry is defined. If multiple entries are defined for
/// a lane, they must be listed in ascending order.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Width {
    /// start position (s-coordinate) relative to the position of the preceding
    /// laneSection record
    pub s_offset: f64,
    /// m ]-∞,∞[ parameter A, offset at s=0
    pub a: f64,
    /// 1 ]-∞,∞[ parameter B
    pub b: f64,
    /// 1/m ]-∞,∞[ parameter C
    pub c: f64,
    /// 1/m² ]-∞,∞[ parameter D
    pub d: f64,
}

/// 5.3.7.2.1.1.4 Road Mark Record
///
/// Each lane within a road cross section can be provided with several road mark
/// entries. The road mark information defines the style of the line at the
/// lane’s outer border. For left lanes, this is the left border, for right
/// lanes the right one. The style of the line separating left and right lanes
/// is determined by the road mark entry for lane zero (i.e. the center lane)
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoadMark {
    /// m [0,∞[ start position (s-coordinate) relative to the position of the preceding laneSection record
    pub s_offset: f64,
    ///- see 6.2 type of the road mark
    #[serde(rename = "type")]
    pub road_mark_type: RoadMarkType,
    /// - see 6.3 weight of the road mark
    pub weight: RoadMarkWeight,
    /// - see 6.4 color of the road mark
    pub color: RoadMarkColor,
    /// - string material of the road mark (identifiers to be defined, use "standard" for the moment.
    pub material: Option<String>,
    /// m [0,∞[ width of the road mark – optional
    #[serde(default)]
    pub width: f64,
    /// [increase decrease both none] allow a lane change in the indicated direction taking into account that lanes are numbered in ascending order from right to left. If the attribute is missing, “both” is assumed to be valid.
    #[serde(default)]
    pub lane_change: LaneChangeType,
    /// m ]-∞,∞[ physical distance of top edge of road mark from reference plane of the lane
    #[serde(default)]
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RoadMarkType {
    None,
    Solid,
    Broken,
    /// (for double solid line),
    SolidSolid,
    /// (from inside to outside, exception: center lane - from left to right)
    SolidBroken,
    ///  (from inside to outside, exception: center lane - from left to right)
    BrokenSolid,
    ///  (from inside to outside, exception: center lane - from left to right)
    BrokenBroken,
    BottsDots,
    /// (meaning a grass edge)
    Grass,
    Curb,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RoadMarkWeight {
    Standard,
    Bold,
}
impl Default for RoadMarkWeight {
    fn default() -> RoadMarkWeight {
        RoadMarkWeight::Standard
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RoadMarkColor {
    /// (equivalent to "white")
    Standard,
    Blue,
    Green,
    Red,
    White,
    Yellow,
}
impl Default for RoadMarkColor {
    fn default() -> RoadMarkColor {
        RoadMarkColor::Standard
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LaneChangeType {
    Increase,
    Decrease,
    Both,
    None,
}
impl Default for LaneChangeType {
    fn default() -> LaneChangeType {
        LaneChangeType::Both
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_header() {
        //::init_logger();
        let s = r##"
        <?xml version="1.0" standalone="yes"?>
        <OpenDRIVE>
            <header revMajor="1" revMinor="1" date="Tue Mar 11 08:53:30 2014" name="test" version="1.00" maxRoad="3" maxJunc="0" maxPrg="0" north="0.0000000000000000e+00" south="1.0000000000000000e+00" east="2.0000000000000000e+00" west="3.0000000000000000e+00" ></header>
        </OpenDRIVE>
        "##;
        use chrono::{TimeZone, Utc};
        use serde_xml_rs::from_str;
        let od: ::Root = from_str(s).unwrap();
        //println!("{:#?}", od);
        assert_eq!(
            od.header,
            ::Header {
                rev_major: 1,
                rev_minor: 1,
                date: Utc.ymd(2014, 3, 11).and_hms(8, 53, 30),
                name: "test".to_string(),
                version: "1.00".to_string(),
                max_road: 3,
                max_junc: 0,
                max_prg: 0,
                north: 0.0,
                south: 1.0,
                east: 2.0,
                west: 3.0,
            }
        );
    }

    #[test]
    fn test_road1() {
        let s = r##"
        <OpenDRIVE>
            <header revMajor="1" revMinor="1" name="" version="1.00" date="Tue Mar 11 08:53:30 2014" north="0.0000000000000000e+00" south="0.0000000000000000e+00" east="0.0000000000000000e+00" west="0.0000000000000000e+00" maxRoad="3" maxJunc="0" maxPrg="0"></header>
                <road name="test_road" length="1.0478122375188772e+02" id="1" junction="-1">
                    <planView></planView>
                </road>
        </OpenDRIVE>
        "##;
        use serde_xml_rs::from_str;
        let od: ::Root = from_str(s).unwrap();
        //println!("{:#?}", od);
        assert_eq!(od.roads.len(), 1);
        assert_eq!(
            od.roads[0],
            ::Road {
                name: "test_road".to_string(),
                length: 1.0478122375188772e+02,
                id: 1,
                junction: -1,
                link: None,
                plan_view: Default::default(),
                //elevation_profile: None,
                //lateral_profile: None
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_road2() {
        let s = r##"
        <road name="test_road" length="1.0478122375188772e+02" id="1" junction="-1">
            <link>
                <successor elementType="road" elementId="3" contactPoint="start"/>
            </link>
            <planView>
                <geometry s="0.0" x="-7.2e+01" y="-5.1e+00" hdg="7.9e-03" length="1.04e+02"> 
                    <line />
                </geometry>
            </planView>
        </road>
        "##;
        use serde_xml_rs::from_str;
        let road: ::Road = from_str(s).unwrap();
        //println!("{:#?}", od);
        assert!(road.link.is_some());
        assert_eq!(
            *road.link.as_ref().unwrap(),
            ::Link {
                predecessor: None,
                successor: Some(::LinkElement {
                    element_type: ::ElementType::Road,
                    element_id: 3,
                    contact_point: ::ContactPoint::Start,
                }),
                neighbors: vec![],
            }
        );
        assert_eq!(road.plan_view.geometries.len(), 1);
        assert_eq!(
            road.plan_view.geometries[0],
            ::Geometry {
                s: 0.0,
                x: -7.2e1,
                y: -5.1,
                hdg: 7.9e-03,
                length: 1.04e+02,
                element: ::GeometryElement::Line,
            }
        );
    }

    #[test]
    fn test_lanes1() {
        let s = r##"
        <lanes>
            <laneOffset s="0.0" a="0.1" b="0.2" c="0.3" d="0.4" />
            <laneSection s="0.0e+00">
                <left>
                    <lane id="1" type="driving" level="0">
                        <link></link>
                        <width sOffset="0.0e+00" a="3.25e+00" b="0.0e+00" c="0.0e+00" d="0.0e+00"/>
                        <roadMark sOffset="0.0e+00" type="solid" weight="standard" color="standard" width="1.3e-01"/>
                    </lane>
                </left>
                <center>
                    <lane id="0" type="driving" level="0">
                        <link></link>
                        <roadMark sOffset="0.0e+00" type="broken" weight="standard" color="standard" width="1.3e-01"/>
                    </lane>
                </center>
                <right>
                    <lane id="-1" type="driving" level="0">
                        <link>
                            <successor id="-1"/>
                        </link>
                        <width sOffset="0.0e+00" a="3.25e+00" b="0.0e+00" c="0.0e+00" d="0.0e+00"/>
                        <roadMark sOffset="0.0e+00" type="solid" weight="standard" color="standard" width="1.3e-01"/>
                    </lane>
                </right>
            </laneSection>
        </lanes>
        "##;
        use serde_xml_rs::from_str;
        let lanes: ::Lanes = from_str(s).unwrap();
        //println!("{:#?}", lanes);
        assert_eq!(lanes.lane_offsets.len(), 1);
        assert_eq!(
            lanes.lane_offsets[0],
            ::LaneOffset {
                s: 0.0,
                a: 0.1,
                b: 0.2,
                c: 0.3,
                d: 0.4
            }
        );
        assert_eq!(lanes.lane_sections.len(), 1);
        assert_eq!(
            lanes.lane_sections[0].left,
            Some(::LaneSubSection {
                lane: vec![::Lane {
                    id: 1,
                    lane_type: ::LaneType::Driving,
                    level: false,
                    link: Some(Default::default()),
                    widths: vec![::Width {
                        s_offset: 0.0,
                        a: 3.25,
                        b: 0.0,
                        c: 0.0,
                        d: 0.0,
                    }],
                    road_marks: vec![::RoadMark {
                        s_offset: 0.0,
                        road_mark_type: ::RoadMarkType::Solid,
                        weight: ::RoadMarkWeight::Standard,
                        color: ::RoadMarkColor::Standard,
                        material: None,
                        width: 0.13,
                        lane_change: ::LaneChangeType::Both,
                        height: 0.0,
                    }],
                }],
            })
        );
    }

    #[test]
    fn test_file1() {
        use std::fs::File;
        use serde_xml_rs;
        let path = "CulDeSac.xodr";
        let file = File::open(&path).unwrap();
        let root: ::Root = serde_xml_rs::from_reader(&file).unwrap();
        println!("{:#?}", root);
    }
}
