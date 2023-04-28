fn main() {
    tonic_build::configure().build_server(false).compile(&["proto/deploy.proto"], &["proto"]).unwrap();
}
