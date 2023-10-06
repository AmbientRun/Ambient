fn main() {
    let fname = std::path::Path::new("assets/Zombie1.x.zip");
    let file = std::fs::File::open(fname).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    archive.extract("assets/").unwrap();
}
