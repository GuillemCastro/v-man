extern crate reqwest;
extern crate tar;

use std::{fs::File, path::Path, io};
use std::env;

use serde::{Serialize, Deserialize};
use color_eyre::eyre::{Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigFile {
    pub name: String,
    pub cpus: Option<u32>,
    pub memory: Option<u64>,
    pub disk: Option<String>,
    pub iso: Option<String>,
    pub vagrant_box: Option<VagrantBox>,
    pub connection_uri: Option<String>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VagrantBox {
    user: String,
    box_name: String,
    version: String    
}

impl VagrantBox {

    pub fn download<P: AsRef<Path>>(&self, destination: P) -> Result<()> {
        let url = format!("https://app.vagrantup.com/{}/boxes/{}/versions/{}/providers/libvirt.box", self.user, self.box_name, self.version);
        let mut resp = reqwest::blocking::get(url.as_str())?;
        let box_file = env::temp_dir().join("vagrant.box");
        let mut out = File::create(box_file)?;
        io::copy(&mut resp, &mut out)?;
        let mut archive = tar::Archive::new(out);
        archive.unpack(destination)?;
        Ok(())
    }

}