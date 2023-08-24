use anyhow::Context;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use toml::map::Map;
use toml::Value;

pub fn import_audio(path: PathBuf, convert: bool) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir().context("Error getting current directory")?;
    let asset_folder_path = current_dir.join("assets");
    let tomlpath = current_dir.join("assets/pipeline.toml");

    if !Path::new(&asset_folder_path).exists() {
        std::fs::create_dir_all(&asset_folder_path)?;
    }
    if !Path::new(&tomlpath).exists() {
        File::create(&tomlpath)?;
    }

    let mut file = File::open(&tomlpath)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = match toml::from_str(&contents) {
        Ok(v) => v,
        Err(_) => Value::Table(Map::new()), // if we cannot parse the file, start with a fresh table
    };

    if let Value::Table(table) = &mut data {
        let pipelines = match table.get_mut("pipelines") {
            Some(Value::Array(arr)) => arr,
            _ => {
                table.insert("pipelines".to_string(), Value::Array(Vec::new()));
                match table.get_mut("pipelines") {
                    Some(Value::Array(arr)) => arr,
                    _ => panic!("Unexpected state"),
                }
            }
        };

        let mut new_pipeline = Map::new();
        new_pipeline.insert(String::from("type"), Value::String(String::from("Audio")));
        new_pipeline.insert(String::from("convert"), Value::Boolean(convert));
        let filename_with_ext = path.file_name().unwrap().to_str().unwrap().to_string();
        new_pipeline.insert(
            String::from("sources"),
            Value::Array(vec![Value::String(filename_with_ext)]),
        );
        if pipelines.contains(&Value::Table(new_pipeline.clone())) {
            println!("\n🚨 This audio file is already imported\n");
            return Ok(());
        }
        println!("\n👉 Importing audio...");
        println!("📘 Read more about audio import here:");
        println!("🔗 https://ambientrun.github.io/Ambient/reference/audio.html\n");
        pipelines.push(Value::Table(new_pipeline));
    } else {
        panic!("Expected table at the root of the TOML document");
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(tomlpath)?;
    write!(file, "{}", toml::to_string(&data)?)?;

    let file_name = path.file_name().unwrap(); // get the file name from the path
    let destination = asset_folder_path.join(file_name);
    std::fs::copy(path.clone(), destination).context("Error copying audio file")?;
    Ok(())
}

pub fn import_model(path: PathBuf, collider_from_model: bool) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir().context("Error getting current directory")?;
    let asset_folder_path = current_dir.join("assets");
    let tomlpath = current_dir.join("assets/pipeline.toml");

    if !Path::new(&asset_folder_path).exists() {
        std::fs::create_dir_all(&asset_folder_path)?;
    }
    if !Path::new(&tomlpath).exists() {
        File::create(&tomlpath)?;
    }

    let mut file = File::open(&tomlpath)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = match toml::from_str(&contents) {
        Ok(v) => v,
        Err(_) => Value::Table(Map::new()), // if we cannot parse the file, start with a fresh table
    };

    if let Value::Table(table) = &mut data {
        if collider_from_model {
            let mut collider_pipeline = Map::new();
            collider_pipeline.insert(
                String::from("type"),
                Value::String(String::from("FromModel")),
            );

            table.insert(
                "pipelines.collider".to_string(),
                Value::Table(collider_pipeline),
            );
        }
        let pipelines = match table.get_mut("pipelines") {
            Some(Value::Array(a)) => a,
            _ => {
                table.insert("pipelines".to_string(), Value::Array(Vec::new()));
                match table.get_mut("pipelines") {
                    Some(Value::Array(a)) => a,
                    _ => panic!("Unexpected state"),
                }
            }
        };

        let mut new_pipeline = Map::new();
        new_pipeline.insert(String::from("type"), Value::String(String::from("Models")));

        let filename_with_ext = path.file_name().unwrap().to_str().unwrap().to_string();
        new_pipeline.insert(
            String::from("sources"),
            Value::Array(vec![Value::String(filename_with_ext)]),
        );
        if pipelines.contains(&Value::Table(new_pipeline.clone())) {
            println!("\n🚨 This model file is already imported\n");
            return Ok(());
        }
        println!("\n👉 Importing model...");
        println!("📘 Read more about model import here:");
        println!("🔗 https://ambientrun.github.io/Ambient/reference/asset_pipeline.html\n");
        pipelines.push(Value::Table(new_pipeline));
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(tomlpath)?;
    let toml_string = toml::to_string(&data)?;
    let s = toml_string.replace(r#""pipelines.collider""#, r#"pipelines.collider"#);
    write!(file, "{}", s)?;

    let file_name = path.file_name().unwrap(); // get the file name from the path
    let destination = asset_folder_path.join(file_name);
    std::fs::copy(path.clone(), destination).context("Error copying audio file")?;
    Ok(())
}
