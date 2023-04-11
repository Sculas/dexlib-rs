use dexlib::dex::DexFile;

#[test]
fn test() {
    let src = include_bytes!("classes.dex");
    let dex = DexFile::new(src).unwrap();
    println!("header: {:#?}", dex.header);
}
