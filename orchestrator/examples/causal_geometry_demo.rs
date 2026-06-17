use ndarray::Array1;
use orchestrator::geometry::service::CausalGeometryService;
use orchestrator::geometry::causal_inner_product::CovarianceMatrix;

fn main() {
    let service = CausalGeometryService::new(768);
    println!("Causal Geometry Service initialized!");
}
