extern crate ructe;

use ructe::{Ructe, Result};

fn main() -> Result<()> {
    Ructe::from_env()?.compile_templates("templates")
}