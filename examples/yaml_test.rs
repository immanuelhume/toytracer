use std::env;
use std::fs::write;
use toytracer::{file_exists, pad_filepath};

const YAML: &str = include_str!("yaml_test.yml");

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or("./tmp/from_yaml.ppm".to_string());
    let path = pad_filepath(&path, file_exists);

    println!("output will be written to {}", path);

    let ppm = toytracer::yaml::gen_world(YAML).expect("should render world from yaml");
    write(path, ppm.as_bytes()).expect("can write ppm string to file");
}
