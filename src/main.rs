include!(concat!(env!("OUT_DIR"), "/templates.rs"));

use config::ConfigFile;
use structopt::StructOpt;
use color_eyre::eyre::{Result};
use crate::vm::{VirtualMachine, create_disk, open_viewer};
use virt::connect::Connect;

use std::fs;

mod vm;
mod config;

#[derive(Debug, StructOpt)]
#[structopt(name = "vman", about = "A tool to create and manage QEMU/KVM virtual machines.")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
    #[structopt(long, short, help = "Definition file", default_value="vman.toml")]
    file: String
}

#[derive(StructOpt, Debug)]
enum Command {
    BuildImage {
        #[structopt(long, short, help = "ISO file to build the VM from")]
        iso: String,
        #[structopt(long, help = "Size of the resulting image in MB", default_value = "20480")]
        disk_size: u32,
        #[structopt(long, short, help = "Destination of the image", default_value = "./disk.img")]
        destination: String,
    },
    Provision,
    Up,
    Down
}

pub fn get_vm(file: String) -> Result<VirtualMachine> {
    let contents = fs::read_to_string(file)?;
    let config_file: ConfigFile = toml::from_str(contents.as_str())?;
    let vm = VirtualMachine::from(config_file);
    Ok(vm)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let opt = Opt::from_args();
    let conn = Connect::open("")?;
    match opt.cmd {
        Command::BuildImage { iso, disk_size, destination } => {
            create_disk(disk_size, destination.as_str())?;
            let dest = fs::canonicalize(destination)?.into_os_string().into_string().unwrap();
            let mut vm = VirtualMachine::from_iso(conn, iso.as_str(), Some(dest.as_str()))?;
            vm.boot()?;
            open_viewer(vm.name())?;
        }
        Command::Provision => {
            let mut vm = get_vm(opt.file)?;
            vm.define()?;
        }
        Command::Up => {
            let mut vm = get_vm(opt.file)?;
            vm.boot();
        }
        Command::Down => {
            let vm = get_vm(opt.file)?;
            vm.shutdown();
        }
    }
    Ok(())
}
