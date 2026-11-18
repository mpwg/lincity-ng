use rustcity_core::{City, Commodity, Point};

#[test]
fn last_day_of_year_posts_monthly_bills_before_taxes() {
    let xml = r#"<lc-game ldsv-version="2140"><globals><total_money>1000</total_money>
    <total_time>1197</total_time><people_pool>7</people_pool><taxable><labor>500</labor><coal>10</coal><goods>200</goods><trade_ex>26000</trade_ex></taxable>
    <money_rates><dole_rate>25</dole_rate></money_rates>
    <stats><population><unemployed_mtd>24</unemployed_mtd><unnat_deaths_mtd>1</unnat_deaths_mtd></population></stats>
    </globals><map size="32"/></lc-game>"#;
    let mut city = City::load(xml.as_bytes()).unwrap();
    city.tick().unwrap();
    assert_eq!(city.balance(), 1000);
    city.tick().unwrap();
    assert_eq!(city.balance(), 26451);
    let mut save = Vec::new();
    city.save(&mut save).unwrap();
    let mut loaded = City::load(save.as_slice()).unwrap();
    loaded.tick().unwrap();
    assert_eq!(loaded.balance(), 26451);
}

#[test]
fn port_pays_imports_and_accumulates_fractional_export_revenue() {
    let xml = r#"<lc-game ldsv-version="2140"><globals><total_money>500</total_money><total_time>97</total_time></globals>
    <map size="32"><Construction group="17" map-x="8" map-y="8">
    <labor>300</labor><goods>999</goods><take_food>1</take_food><give_goods>1</give_goods></Construction></map></lc-game>"#;
    let mut city = City::load(xml.as_bytes()).unwrap();
    let p = Point { x: 8, y: 9 };
    city.tick().unwrap();
    assert_eq!(city.stock(p, Commodity::Food), Some(5000));
    assert_eq!(city.stock(p, Commodity::Goods), Some(500));
    assert_eq!(city.stock(p, Commodity::Labor), Some(200));
    assert_eq!(city.balance(), 450);
    city.tick().unwrap();
    assert_eq!(city.balance(), 425);
    assert_eq!(city.stock(p, Commodity::Food), Some(7500));
}

#[test]
fn staffed_market_receives_distant_farm_stock_only_when_buying() {
    let xml = r#"<lc-game ldsv-version="2140"><globals/><map size="32">
    <Construction group="6" map-x="4" map-y="4"><labor>2</labor></Construction>
    <Construction group="5" map-x="9" map-y="4"><food>5500</food></Construction></map></lc-game>"#;
    let tape = b"LINCITY-SIMULATION-TRACE 1
C 1858042402
C 893031963
P 1 4 4 6
P 2 9 4 5
D 1 2 2 1
D 2 2 1 2
Z
";
    let decisions =
        rustcity_core::decisions::Decisions::replay(std::io::Cursor::new(tape)).unwrap();
    let mut city = City::load_with_decisions(xml.as_bytes(), decisions).unwrap();
    let p = Point { x: 5, y: 5 };
    assert!(city.set_market_trade(p, Commodity::Food, false, true));
    city.tick().unwrap();
    assert_eq!(city.stock(p, Commodity::Food), Some(0));
    assert!(city.set_market_trade(p, Commodity::Food, true, false));
    city.tick().unwrap();
    assert_eq!(city.stock(p, Commodity::Food), Some(5310));
    assert_eq!(city.stock(p, Commodity::Labor), Some(0));
}
