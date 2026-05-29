module "oci" {
  source = "./modules/oci"
  region = var.oci_region
  compartment_id = var.oci_compartment
}

module "aws" {
  source = "./modules/aws"
  region = var.aws_region
}
