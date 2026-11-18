use rustcity_core::Commodity;

#[test]
fn canonical_names_match_the_cpp_reference_transcript() {
    let transcript = include_str!("../../lincity-core/parity/cases/v1/expected.txt");
    let mut records = transcript
        .lines()
        .skip_while(|line| *line != "names")
        .skip(1);
    for (index, commodity) in Commodity::ALL.into_iter().enumerate() {
        let (name, id) = records.next().unwrap().split_once(' ').unwrap();
        assert_eq!(id.parse::<usize>().unwrap(), index);
        assert_eq!(commodity.save_name(), name);
        assert_eq!(Commodity::from_save_name(name), Some(commodity));
    }
    for invalid in ["Food", "", "unknown"] {
        assert_eq!(Commodity::from_save_name(invalid), None);
    }
}
