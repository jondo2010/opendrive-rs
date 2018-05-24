use proj5::prelude::*;

#[test]
fn test_proj() {
    let ellipsoid = WGS_1984_ELLIPSOID;

    let src = CoordinateSource::CoordinateBuf(Box::new(CoordinateBuf {
        data: vec![
            (8.044918800000e+1, 2.488364150000e+2),
            (8.064432870000e+1, 2.488880630000e+2),
            (8.098982890000e+1, 2.489818280000e+2),
            (8.144194620000e+1, 2.491042410000e+2),
        ],
        crs: Box::new(MercatorSystem {}),
        ellipsoid: ellipsoid,
    }));

    let mut tgt = CoordinateSource::LonLatBuf(Box::new(LonLatBuf {
        data: Vec::new(),
        ellipsoid: ellipsoid,
    }));

    let mut strategy = MultithreadingStrategy::SingleCore;
    src.project(&mut tgt, &mut strategy);
    println!("first batch of coordinates: {:#?}", tgt.get_data_ref());
}
