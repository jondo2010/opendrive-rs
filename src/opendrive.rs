use crate::errors;
use crate::parse_util;
use chrono;
use lyon_geom;
use lyon_path;

use serde::{de, Deserialize, Serialize};

pub mod units {
    pub struct Meter;
}

//pub mod types;
pub use crate::types;

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
    /// 1 major revision number of OpenDRIVE® format
    pub rev_major: u16,
    /// 4 minor revision number of OpenDRIVE® format
    pub rev_minor: u16,
    /// database name
    pub name: String,
    /// version number of this database (format: a.bb)
    pub version: f32,
    #[serde(with = "parse_util::odr_dateformat", default = "Header::default_date")]
    /// time/date of database creation according to ISO 8601 (preference: YYYY-MM-DDThh:mm:ss)
    pub date: chrono::DateTime<chrono::Utc>,
    /// maximum inertial y value [m]
    pub north: f64,
    /// minimum inertial y value [m]
    pub south: f64,
    /// maximum inertial x value [m]
    pub east: f64,
    /// minimum inertial x value [m]
    pub west: f64,

    pub max_road: u32,
    pub max_junc: u32,
    pub max_prg: u32,

    /// vendor string - - vendor name
    //pub vendor: String,
    pub geo_reference: Option<GeoReference>,
}
impl Header {
    fn default_date() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

/// The information for geographic reference of a database may be provided as
/// child node to the OpenDRIVE header node. It will provide all information
/// necessary to convert OpenDRIVE's cartesian x/y/z co-ordinates into a
/// corresponding geographic reference system. There must be no more than one
/// definition of the geographic projection.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeoReference {
    #[serde(rename = "$value")]
    pub projection: String,
}

/// 5.3.1 Road Header Record
/// The road header record defines the basic parameters of an individual road.
/// It is followed immediately by other records defining geometry and logical
/// properties of the road.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "road", rename_all = "camelCase")]
pub struct Road {
    /// name of the road
    pub name: String,
    /// total length of the reference line in the xy-plane
    pub length: types::Length,
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
impl Road {
    pub fn validate(&self) -> Result<(), errors::ValidationError> {
        //self.plan_view.validate()?;
        let sum_length = self.plan_view.sum_length();
        if self.length != sum_length {
            return Err(errors::ValidationError::ReferenceLineLength((
                self.length,
                sum_length,
            )));
        }

        Ok(())
    }
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

impl PlanView {
    pub fn validate(&self) -> Result<(), errors::ValidationError> {
        use super::Monotonic;
        use lyon_geom::Segment;

        // The s values in each Geometry element must monotonically increase
        if !self.geometries.iter().map(|g| g.s).is_monotonic() {
            return Err(errors::ValidationError::ReferenceLineGeometry(
                "s values in geometry not monotonic",
            ));
        }

        // Ensure that the start and end points of each segment match
        let s = self.geometries.iter().map(|g| g.as_segment());
        if !s
            .clone()
            .zip(s.skip(1))
            .by_ref()
            .all(|(a, b)| a.to() == b.from())
        {
            return Err(errors::ValidationError::ReferenceLineGeometry(
                "not all start and end points align",
            ));
        }

        Ok(())
    }

    /// Sum up the lengths of all Geometry elements
    pub fn sum_length(&self) -> types::Length {
        types::Length::new(self.geometries.iter().fold(0.0, |acc, g| g.length.0 + acc))
    }
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
    pub s: types::Length,
    /// m ]-∞,∞[ start position (x inertial)
    pub x: types::Length,
    /// m ]-∞,∞[ start position (y inertial)
    pub y: types::Length,
    /// rad ]-∞,∞[ start orientation (inertial heading)
    #[serde(with = "parse_util::angle")]
    pub hdg: types::Angle,
    /// m [0,∞[ length of the element's reference line
    pub length: types::Length,
    #[serde(rename = "$value")]
    pub element: GeometryElement,
}
impl Geometry {
    /// Convert the OpenDRIVE types into a type that implements `lyon_geom::Segment`
    pub fn as_segment(&self) -> types::Segment<f64> {
        use euclid;
        match self.element {
            GeometryElement::Line => {
                let start = euclid::TypedPoint2D::from_lengths(self.x, self.y);
                let rot = types::Rotation::new(self.hdg);
                let v = euclid::TypedVector2D::from_lengths(self.length, types::Length::new(0.0));
                let end = start + rot.transform_vector(&v);
                types::Segment::Line(lyon_geom::LineSegment {
                    from: start,
                    to: end,
                })
            }
            GeometryElement::Spiral {
                curv_start: _,
                curv_end: _,
            } => types::Segment::Line(lyon_geom::LineSegment {
                from: euclid::point2(0.0, 0.0),
                to: euclid::point2(0.0, 0.0),
            }),
            GeometryElement::Arc { curvature } => {
                let radius = 1.0 / curvature;
                let start_angle = self.hdg;
                let sweep_angle = types::Angle::radians((self.length * curvature).get());

                // Find the center by rotating the vector (0.0, radius) by the heading
                let center = types::Rotation::new(self.hdg)
                    .transform_point(&euclid::point2(0.0, radius))
                    + euclid::TypedVector2D::from_lengths(self.x, self.y);

                types::Segment::Arc(lyon_geom::Arc {
                    center: center,
                    radii: euclid::vec2(radius, radius),
                    start_angle: start_angle,
                    sweep_angle: sweep_angle,
                    x_rotation: -euclid::Angle::frac_pi_2(), // OpenDRIVE zero-heading is pi/2 rotated from the lyon_geom::Arc
                })
            }
            GeometryElement::Poly3 {
                a: _,
                b: _,
                c: _,
                d: _,
            } => types::Segment::Line(lyon_geom::LineSegment {
                from: euclid::point2(0.0, 0.0),
                to: euclid::point2(0.0, 0.0),
            }),
        }
    }
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
    #[serde(with = "parse_util::flexible_boolean")]
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
            id: 0,
            lane_type: LaneType::None,
            level: false,
            link: None,
            widths: vec![],
            road_marks: vec![],
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
