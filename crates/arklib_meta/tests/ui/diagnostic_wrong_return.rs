use arklib_meta::diagnostic;

pub struct TaskVector;

#[diagnostic]
pub fn my_diag() -> TaskVector {
    TaskVector
}

fn main() {}
