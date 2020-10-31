use std::{ffi::OsStr, path::Path};

use virt::domain::{Domain, VIR_DOMAIN_NONE};
use virt::connect::Connect;
use color_eyre::eyre::{Result, eyre};
use crate::templates::{build_iso_xml};

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