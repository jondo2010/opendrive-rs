#[test]
fn test_monotonic() {
    use super::is_monotonic;

    let x = vec![1, 2, 3, 4, 5];
    assert!(is_monotonic(&x));

    let x = vec![1, 2, 3, 4, 0];
    assert!(!is_monotonic(&x));

    let x = vec![0.0, 0.1234, 0.54, -0.32];
    assert!(!is_monotonic(&x));
}

#[cfg(test)]
mod parsing {
    use opendrive;

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
        let od: opendrive::Root = from_str(s).unwrap();
        //println!("{:#?}", od);
        assert_eq!(
            od.header,
            opendrive::Header {
                rev_major: 1,
                rev_minor: 1,
                date: Utc.ymd(2014, 3, 11).and_hms(8, 53, 30),
                name: "test".to_string(),
                version: 1.0,
                max_road: 3,
                max_junc: 0,
                max_prg: 0,
                north: 0.0,
                south: 1.0,
                east: 2.0,
                west: 3.0,
                //vendor: "".to_string(),
                geo_reference: None,
            }
        );
    }

    #[test]
    fn test_geo_reference() {
        let s = r##"
        <geoReference>
            <![CDATA[+proj=utm +zone=32 +ellps=WGS84 +datum=WGS84 +units=m +no_defs]]>
        </geoReference>
        "##;
        use serde_xml_rs::from_str;
        let geo: opendrive::GeoReference = from_str(s).unwrap();
        println!("{:#?}", geo);
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
        let od: opendrive::Root = from_str(s).unwrap();
        //println!("{:#?}", od);
        assert_eq!(od.roads.len(), 1);
        assert_eq!(
            od.roads[0],
            opendrive::Road {
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
        let road: opendrive::Road = from_str(s).unwrap();
        //println!("{:#?}", od);
        assert!(road.link.is_some());
        assert_eq!(
            *road.link.as_ref().unwrap(),
            opendrive::Link {
                predecessor: None,
                successor: Some(opendrive::LinkElement {
                    element_type: opendrive::ElementType::Road,
                    element_id: 3,
                    contact_point: opendrive::ContactPoint::Start,
                }),
                neighbors: vec![],
            }
        );
        assert_eq!(road.plan_view.geometries.len(), 1);
        assert_eq!(
            road.plan_view.geometries[0],
            opendrive::Geometry {
                s: 0.0,
                x: -7.2e1,
                y: -5.1,
                hdg: 7.9e-03,
                length: 1.04e+02,
                element: opendrive::GeometryElement::Line,
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
        let lanes: opendrive::Lanes = from_str(s).unwrap();
        //println!("{:#?}", lanes);
        assert_eq!(lanes.lane_offsets.len(), 1);
        assert_eq!(
            lanes.lane_offsets[0],
            opendrive::LaneOffset {
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
            Some(opendrive::LaneSubSection {
                lane: vec![
                    opendrive::Lane {
                        id: 1,
                        lane_type: opendrive::LaneType::Driving,
                        level: false,
                        link: Some(Default::default()),
                        widths: vec![
                            opendrive::Width {
                                s_offset: 0.0,
                                a: 3.25,
                                b: 0.0,
                                c: 0.0,
                                d: 0.0,
                            },
                        ],
                        road_marks: vec![
                            opendrive::RoadMark {
                                s_offset: 0.0,
                                road_mark_type: opendrive::RoadMarkType::Solid,
                                weight: opendrive::RoadMarkWeight::Standard,
                                color: opendrive::RoadMarkColor::Standard,
                                material: None,
                                width: 0.13,
                                lane_change: opendrive::LaneChangeType::Both,
                                height: 0.0,
                            },
                        ],
                    },
                ],
            })
        );
    }

    #[test]
    fn test_file1() {
        use serde_xml_rs;
        use std::fs::File;
        let path = "CulDeSac.xodr";
        let file = File::open(&path).unwrap();
        let root: opendrive::Root = serde_xml_rs::from_reader(&file).unwrap();
        //println!("{:#?}", root);
    }
}
