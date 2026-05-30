use arklib_meta::intervention;

pub type LayerId = usize;

#[intervention]
pub fn my_intervention() -> Result<(), ()> {
    Ok(())
}

fn main() {}
