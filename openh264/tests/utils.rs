use openh264::nal_units;

#[test]
fn split_at_nals() {
    let src = &include_bytes!("data/multi_512x512.h264")[..];
    let slices = nal_units(src).collect::<Vec<_>>();

    assert_eq!(slices[0].len(), 21);
    assert_eq!(slices[1].len(), 9);
    assert_eq!(slices[2].len(), 2736);
    assert_eq!(slices[3].len(), 2688);
    assert_eq!(slices[4].len(), 2672);
    assert_eq!(slices[5].len(), 2912);
    assert_eq!(slices[6].len(), 3214);
}
