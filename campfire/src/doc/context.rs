use rustdoc_types as rdt;
use std::{collections::HashMap, path::Path};

pub struct Context {
    pub crates: HashMap<String, rdt::Crate>,
    pub path_to_crate_and_id: HashMap<String, (String, rdt::Id)>,
}
impl Context {
    pub fn new(crates: &[&Path]) -> anyhow::Result<Self> {
        let crates: HashMap<String, rdt::Crate> = crates
            .iter()
            .map(|n| {
                let build = rustdoc_json::Builder::default()
                    .toolchain("nightly")
                    .document_private_items(true)
                    .manifest_path(n)
                    .silent(true)
                    .build()
                    .unwrap();

                let krate = serde_json::from_str(&std::fs::read_to_string(build).unwrap()).unwrap();
                (n.to_string_lossy().to_string(), krate)
            })
            .collect();

        let path_to_crate_and_id = crates
            .iter()
            .flat_map(|(n, krate)| {
                krate
                    .paths
                    .iter()
                    .filter(|p| p.1.crate_id == 0)
                    .map(|p| (p.1.path.join("::"), (n.clone(), p.0.clone())))
            })
            .collect();

        Ok(Self {
            crates,
            path_to_crate_and_id,
        })
    }

    pub fn get(&self, path: &str) -> Option<(&rdt::Crate, &rdt::Item)> {
        let (crate_path, id) = self.path_to_crate_and_id.get(path)?;
        let krate = self.crates.get(crate_path)?;
        Some((krate, krate.index.get(id)?))
    }

    pub fn get_by_path<'a>(
        &'a self,
        source_crate: &'a rdt::Crate,
        path: &rdt::Path,
    ) -> Option<(&rdt::Crate, &rdt::Item)> {
        match source_crate.index.get(&path.id) {
            Some(i) => Some((source_crate, i)),
            None => self.get(&source_crate.paths.get(&path.id)?.path.join("::")),
        }
    }
}
