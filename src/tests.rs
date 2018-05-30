#[test]
fn test_monotonic() {
    use super::Monotonic;

    let x = vec![1, 2, 3, 4, 5];
    assert!(x.iter().is_monotonic());

    let x = vec![1, 2, 3, 4, 0];
    assert!(!x.iter().is_monotonic());

    let x = vec![0.0, 0.1234, 0.54, -0.32];
    assert!(!x.iter().is_monotonic());
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
                length: opendrive::types::Length::new(1.0478122375188772e+02),
                id: 1,
                junction: -1,
                link: None,
                plan_view: Default::default(),
                elevation_profile: None,
                lateral_profile: None,
                lanes: None,
            }
        );
    }

    #[test]
    fn test_geometry() {
        let s = r##"
            <geometry s="0.0" x="3.1936295054484493e+01" y="-4.3587594300083827e+00" hdg="7.9998293398869432e-03" length="2.0721630948442136e+00"><line/></geometry>
        "##;
        use serde_xml_rs::from_str;
        let geo: opendrive::Geometry = from_str(s).unwrap();
        assert_eq!(geo.hdg, opendrive::types::Angle::radians(7.9998293398869432e-03));
    }

    #[test]
    fn test_road2() {
        let s = r##"
        <road name="test_road" length="1.0478122375188772e+02" id="1" junction="-1">
            <link>
                <successor elementType="road" elementId="3" contactPoint="start"/>
            </link>
            <planView>
                <geometry s="0.0" x="-7.2e+01" y="-5.1e+00" hdg="7.9e-03" length="1.04e+02"><line /></geometry>
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
                s: opendrive::types::Length::new(0.0),
                x: opendrive::types::Length::new(-7.2e1),
                y: opendrive::types::Length::new(-5.1),
                //hdg: euclid::Angle<f64>{7.9e-03},
                hdg: opendrive::types::Angle::radians(7.9e-03),
                length: opendrive::types::Length::new(1.04e+02),
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
    fn test_planview() {
        let s = r##"
        <planView>
            <geometry s="0.0" x="3.1936295054484493e+01" y="-4.3587594300083827e+00" hdg="7.9998293398869432e-03" length="2.0721630948442136e+00">
                <line/>
            </geometry>
            <geometry s="2.0721630948442136e+00" x="3.4008391843292344e+01" y="-4.3421826556979193e+00" hdg="7.9998293398872988e-03" length="5.7125709578755304e+00">
                <spiral curvStart="-0.0" curvEnd="-5.9114369622925669e-02"/>
            </geometry>
            <geometry s="7.7847340527197435e+00" x="3.9707082592335631e+01" y="-4.6174670591660050e+00" hdg="-1.6084768620939371e-01" length="6.0636650000000021e+00">
                <arc curvature="-5.9114369622925669e-02"/>
            </geometry>
            <geometry s="1.3848399052719746e+01" x="4.5392930162567524e+01" y="-6.6292251856718263e+00" hdg="-5.1929742028899106e-01" length="5.7125709578755304e+00">
                 <spiral curvStart="-5.9114369622925669e-02" curvEnd="-0.0"/>
            </geometry>
            <geometry s="1.9560970010595277e+01" x="4.9996684586531416e+01" y="-9.9991366028133708e+00" hdg="-6.8814493584077607e-01" length="8.9362262700839490e-01">
                <line/>
            </geometry>
            <geometry s="2.0454592637603671e+01" x="5.0686941495122845e+01" y="-1.0566681138196444e+01" hdg="-6.8814493584075453e-01" length="6.2066121627724709e+00">
                <spiral curvStart="0.0" curvEnd="8.0978518504482233e-02"/>
            </geometry>
            <geometry s="2.6661204800376140e+01" x="5.5779610165223573e+01" y="-1.4083929379520304e+01" hdg="-4.3684380690538838e-01" length="5.0000000000000014e+01">
                <arc curvature="8.0978518504482233e-02"/>
            </geometry>
            <geometry s="7.6661204800376154e+01" x="5.5406167361537790e+01" y="8.1125499406994486e+00" hdg="-2.6711031888558949e+00" length="6.2066121627724709e+00">
                 <spiral curvStart="8.0978518504482233e-02" curvEnd="0.0000000000000000e+00"/>
            </geometry>
            <geometry s="8.2867816963148627e+01" x="5.0434698751727190e+01" y="4.4259784662547794e+00" hdg="-2.4198020599242533e+00" length="5.7004576686428177e+00">
                <line/>
            </geometry>
            <geometry s="8.8568274631791439e+01" x="4.6155799357937703e+01" y="6.5951625221137222e-01" hdg="-2.4198020599242529e+00" length="1.7765358576322927e+00">
                <spiral curvStart="-0.0000000000000000e+00" curvEnd="-5.6494314851365926e-02"/>
            </geometry>
            <geometry s="9.0344810489423736e+01" x="4.4802993754335226e+01" y="-4.9169626144549478e-01" hdg="-2.4699841479659219e+00" length="1.0858200331373734e+01">
                <arc curvature="-5.6494314851365926e-02"/>
            </geometry>
            <geometry s="1.0120301082079747e+02" x="3.4817973688592701e+01" y="-4.3059936354469119e+00" hdg="-3.0834107362057552e+00" length="1.7765358576322927e+00">
                 <spiral curvStart="-5.6494314851365926e-02" curvEnd="-0.0000000000000000e+00"/>
            </geometry>
            <geometry s="1.0297954667842977e+02" x="3.3042179669337365e+01" y="-4.3499123530895600e+00" hdg="-3.1335928242499071e+00" length="1.1059200025943670e+00">
                <line/>
            </geometry>
        </planView>
        "##;
        use serde_xml_rs::from_str;
        let plan_view: opendrive::PlanView = from_str(s).unwrap();
        assert_eq!(plan_view.validate().unwrap(), ());
        println!("Length: {}", plan_view.sum_length());

        let g = &plan_view.geometries[0];
        g.as_segment().
        
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
