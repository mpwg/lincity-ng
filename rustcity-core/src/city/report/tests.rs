use super::*;

#[test]
fn reports_match_native_rows_for_every_registered_type() {
    let mut active_sprites = include_str!("active_sprites.txt").lines();
    for (reference, stocked, working) in [
        (include_str!("reference.tsv"), false, false),
        (include_str!("stocked.tsv"), true, false),
        (include_str!("working.tsv"), false, true),
    ] {
        let mut groups = reference.split("report\t").skip(1);
        for kind in BuildingKind::ALL {
            if working
                && !matches!(
                    kind,
                    BuildingKind::SolarPower
                        | BuildingKind::Substation
                        | BuildingKind::CoalPower
                        | BuildingKind::LightIndustry
                        | BuildingKind::HeavyIndustry
                        | BuildingKind::Tip
                        | BuildingKind::School
                        | BuildingKind::Blacksmith
                        | BuildingKind::Waterwell
                )
            {
                continue;
            }
            let mut city = load(kind, if stocked { 1750000 } else { 0 });
            if stocked {
                let id = city.buildings.at(Point { x: 12, y: 12 }).unwrap();
                let b = city.buildings.get_mut(id).unwrap();
                b.evacuating = true;
                let limits = b.limits();
                for c in Commodity::ALL {
                    let i = c as usize;
                    b.stock[i] = kind.metadata().rule(c).max_load / 3;
                    let (prod, cons) = limits[i];
                    b.production.previous[i] = if cons != 0 { -cons / 3 } else { prod / 2 };
                }
            }
            if working {
                let id = city.buildings.at(Point { x: 12, y: 12 }).unwrap();
                for _ in 0..100 {
                    let b = city.buildings.get_mut(id).unwrap();
                    for c in Commodity::ALL {
                        b.stock[c as usize] = kind.metadata().rule(c).max_load / 2;
                    }
                    city.tick().unwrap();
                }
            }
            if working {
                let count = match kind {
                    BuildingKind::CoalPower => 8,
                    BuildingKind::LightIndustry => 2,
                    _ => 0,
                };
                let mut tape = "LINCITY-SIMULATION-TRACE 1\nA 1000 1 1\n".to_owned();
                use std::fmt::Write;
                for draw in 5..5 + count {
                    writeln!(tape, "C {draw}").unwrap();
                }
                tape.push_str("Z\n");
                city.decisions = Decisions::replay(std::io::Cursor::new(tape)).unwrap();
                city.animate(1000).unwrap();
                assert_eq!(
                    Some(sprite_line(&city, Point { x: 12, y: 12 }, 1).as_str()),
                    active_sprites.next(),
                    "{kind:?}"
                );
            }
            for mode in [ReportMode::Inventory, ReportMode::Production] {
                let block = groups.next().unwrap();
                let (header, expected) = block.split_once('\n').unwrap();
                assert_eq!(
                    header,
                    format!(
                        "{}\t{}",
                        kind.id(),
                        u8::from(mode == ReportMode::Production)
                    )
                );
                let rows = city.building_report(Point { x: 12, y: 12 }, mode).unwrap();
                let actual: Vec<_> = rows
                    .into_iter()
                    .map(|row| match row {
                        ReportRow::Blank => "blank".into(),
                        ReportRow::Label(k) => format!("label\t{k}"),
                        ReportRow::Text(k, v) => format!("text\t{k}\t{v}"),
                        ReportRow::Integer(k, v) => format!("int\t{k}\t{v}"),
                        ReportRow::Decimal(k, v) => format!("decimal\t{k}\t{v}"),
                        ReportRow::Percentage(k, v) => format!("percent\t{k}\t{v}"),
                        ReportRow::Quantity(k, v, max) => format!("qty\t{k}\t{v}\t{max}"),
                        ReportRow::PrefixedQuantity(p, k, v, max) => {
                            format!("pref\t{p}\t{k}\t{v}\t{max}")
                        }
                    })
                    .collect();
                let expected: Vec<_> = expected.lines().collect();
                assert_eq!(actual.len(), expected.len(), "{kind:?} {mode:?}");
                for (a, e) in actual.iter().zip(expected) {
                    if a == e {
                        continue;
                    }
                    let (a1, a2) = a.rsplit_once('\t').unwrap();
                    let (e1, e2) = e.rsplit_once('\t').unwrap();
                    assert_eq!(a1, e1, "{kind:?} {mode:?}");
                    assert!(
                        (a2.parse::<f64>().unwrap() - e2.parse::<f64>().unwrap()).abs() < 1e-6,
                        "{kind:?} {mode:?} {a} != {e}"
                    );
                }
            }
        }
        assert!(groups.next().is_none());
    }
}

#[test]
fn sprites_match_native_groups_before_and_after_animation() {
    let mut expected = include_str!("sprites.txt").lines();
    for kind in BuildingKind::ALL {
        let pos = Point { x: 12, y: 12 };
        let mut city = load(kind, 0);
        for phase in 0..2 {
            if phase == 1 {
                let id = city.buildings.at(pos).unwrap();
                let b = city.buildings.get_mut(id).unwrap();
                for c in Commodity::ALL {
                    b.stock[c as usize] = kind.metadata().rule(c).max_load * 3 / 4;
                }
                city.animate(1000).unwrap();
            }
            let line = sprite_line(&city, pos, phase);
            assert_eq!(Some(line.as_str()), expected.next());
        }
    }
}

fn load(kind: BuildingKind, tech: i32) -> City {
    let water = if matches!(
        kind,
        BuildingKind::TrackBridge | BuildingKind::RoadBridge | BuildingKind::RailBridge
    ) {
        r#"<MapTile group="21" map-x="12" map-y="12"/>"#
    } else {
        ""
    };
    let xml = format!(
        r#"<lc-game ldsv-version="2140"><globals/><map size="40">{water}<Construction group="{}" map-x="12" map-y="12"><tech>{tech}</tech></Construction></map></lc-game>"#,
        kind.id()
    );
    City::load(xml.as_bytes()).unwrap()
}

#[test]
fn solar_and_station_publish_previous_month_power_conversion() {
    let xml = r#"<lc-game ldsv-version="2140"><globals><total_time>98</total_time></globals><map size="40"><Construction group="2" map-x="4" map-y="4"><labor>50</labor></Construction><Construction group="3" map-x="12" map-y="12"><hivolt>1500</hivolt></Construction></map></lc-game>"#;
    let mut city = City::load(xml.as_bytes()).unwrap();
    city.tick().unwrap();
    let solar = city
        .building_report(Point { x: 5, y: 5 }, ReportMode::Production)
        .unwrap();
    assert!(solar.contains(&ReportRow::PrefixedQuantity("<-- ", "Hi-Volt", 900, 90000)));
    assert!(solar.contains(&ReportRow::PrefixedQuantity("--> ", "Labor", -50, -5000)));
    let station = city
        .building_report(Point { x: 12, y: 12 }, ReportMode::Production)
        .unwrap();
    assert!(station.contains(&ReportRow::PrefixedQuantity(
        "<-- ", "Lo-Volt", 3000, 300000
    )));
    assert!(station.contains(&ReportRow::PrefixedQuantity(
        "--> ", "Hi-Volt", -1500, -150000
    )));
}

fn sprite_line(city: &City, pos: Point, phase: i32) -> String {
    let v = city.building_visual(pos).unwrap();
    let kind = v.kind;
    let mut line = format!("{} {}", kind.id(), phase);
    for s in v.layers {
        use std::fmt::Write;
        write!(
            line,
            " {},{},{},{}",
            s.resource, s.frame, s.offset.0, s.offset.1
        )
        .unwrap();
    }
    line
}
