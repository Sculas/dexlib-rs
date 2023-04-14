use dexlib::dex::DexFile;

#[test]
fn test() {
    let src = include_bytes!("classes.dex");
    let dex = DexFile::new(src).unwrap();
    // println!("header: {:#?}", dex.header);
    // println!("map_list: {:#?}", dex.map_list);

    let sidx = dex.strings().len() / 2;
    let sid_1 = dex.strings().id_at(sidx).unwrap();
    let str = dex.strings().get(&sid_1).unwrap();
    let sid_2 = dex.strings().find(&str).unwrap();
    assert_eq!(sid_1, sid_2);

    // for i in 0..dex.strings().len() {
    //     let str = dex.strings().get(dex.strings().id_at(i).unwrap()).unwrap();
    //     if str.starts_with("L") && str.contains("/") && str.ends_with(";") {
    //         println!("string {i}: {str}");
    //         break;
    //     }
    // }
}
