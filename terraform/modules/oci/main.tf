resource "oci_containerengine_cluster" "arkhe" {
  compartment_id = var.compartment_id
  name           = "arkhe-cathedral"
  kubernetes_version = "v1.30"
  vcn_id = oci_core_vcn.arkhe.id
}

resource "oci_core_vcn" "arkhe" {
  compartment_id = var.compartment_id
  cidr_block     = "10.0.0.0/16"
  display_name   = "arkhe-vcn"
}

resource "oci_core_fast_connect" "aws_interconnect" {
  # ... configuração do Oracle Interconnect for AWS
}
