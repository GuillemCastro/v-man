extern crate num_cpus;

use std::{ffi::OsStr, path::Path};

use virt::domain::{Domain, VIR_DOMAIN_NONE};
use virt::connect::Connect;
use color_eyre::eyre::{Result, eyre};

use crate::templates::{build_iso_xml};
use crate::config;

use std::fs;

use std::process::Command;

#[derive(Debug)]
pub struct VirtualMachine {
    xml: Option<String>,
    domain: Option<Domain>,
    conn: Connect,
    properties: VMProperties
}

#[derive(Debug, Clone)]
pub struct VMProperties {
    pub name: String,
    pub cpus: u32,
    pub memory: u64, // Memory in KB
    pub disk: Option<String>,
    pub iso: Option<String>,
}

static VMAN_PATH: &'static str = ".vman";

impl VirtualMachine {

    pub fn name(&self) -> &str {
        return self.properties.name.as_str();
    }

    pub fn from_iso(conn: Connect, iso: &str, disk: Option<&str>) -> Result<Self> {
        let name = Path::new(iso)
            .file_stem().unwrap_or(OsStr::new("default")).to_str().unwrap_or("default");
        let properties = VMProperties {
            name: name.to_owned(),
            cpus: 2,
            memory: 2097152, // 2GB
            disk: disk.map(|s| s.to_owned()),
            iso: Some(iso.to_owned()),
        };
        let mut buf = Vec::new();
        build_iso_xml(&mut buf, properties.clone())?;
        let xml = match String::from_utf8(buf) {
            Ok(res) => res,
            Err(_) => return Err(eyre!("Error creating domain's XML")),
        };
        let vm = VirtualMachine {  
            xml: Some(xml),
            domain: None,
            conn: conn,
            properties: properties.clone()
        };
        Ok(vm)
    }

    pub fn boot(&mut self) -> Result<()> {
        if self.domain.is_none() && self.xml.is_some() {
            let xml = self.xml.as_ref().unwrap().as_str();
            let domain = Domain::create_xml(&self.conn, xml, VIR_DOMAIN_NONE)?;
            self.domain = Some(domain);
        }
        else {
            self.domain.as_ref().unwrap().create()?;
        }
        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        match &self.domain {
            Some(domain) => {
                domain.shutdown()?
            }
            None => return Err(eyre!("VirtualMachine has no active domain"))
        };
        Ok(())
    }

    pub fn define(&mut self) -> Result<()> {
        let domain = match &self.xml {
            Some(xml) => Domain::define_xml(&self.conn, xml)?,
            None => return Err(eyre!("VirtualMachine has no XML"))
        };
        self.domain = Some(domain);
        Ok(())
    }

}

impl From<config::ConfigFile> for VirtualMachine {

    fn from(config: config::ConfigFile) -> Self {
        let cpus = match config.cpus {
            Some(cpus) => cpus,
            None => (num_cpus::get()/2) as u32
        };
        let memory = match config.memory {
            Some(mem) => mem,
            None => 2*1024
        };
        fs::create_dir_all(Path::new(VMAN_PATH).join(&config.name));
        let disks_path = Path::new(VMAN_PATH).join(&config.name).join("disks");
        let disk_path = disks_path.join("disk.img");
        fs::create_dir_all(&disks_path);
        if config.vagrant_box.is_some() {
            config.vagrant_box.unwrap().download(disks_path);
        }
        else if config.disk.is_some() {
            fs::copy(config.disk.unwrap(), &disk_path);
        }
        let properties = VMProperties {
            name: config.name,
            cpus: cpus,
            memory: memory,
            disk: Some(disk_path.as_os_str().to_str().unwrap().to_owned()),
            iso: config.iso
        };
        let mut buf = Vec::new();
        build_iso_xml(&mut buf, properties.clone());
        let xml = String::from_utf8(buf).unwrap();
        let connection_uri = config.connection_uri.unwrap_or(String::from(""));
        let conn = Connect::open(connection_uri.as_str()).unwrap();
        VirtualMachine {
            xml: Some(xml),
            domain: None,
            conn: conn,
            properties: properties
        }
    }

}

pub fn create_disk(size: u32, destination: &str) -> Result<()> {
    let size_arg = format!("{}M", size);
    let out = Command::new("qemu-img")
        .arg("create")
        .arg("-f")
        .arg("qcow2")
        .arg(destination)
        .arg(size_arg)
        .output()?;
    println!("status: {}", out.status);
    Ok(())
}

pub fn open_viewer(name: &str) -> Result<()> {
    let out = Command::new("virt-viewer")
        .arg("--attach")
        .arg(name)
        .output()?;
    println!("status: {}", out.status);
    Ok(())
}