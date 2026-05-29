resource "aws_eks_cluster" "arkhe" {
  name     = "arkhe-cathedral"
  role_arn = aws_iam_role.cluster.arn
  vpc_config {
    subnet_ids = aws_subnet.private[*].id
  }
}

resource "aws_dx_connection" "oci_interconnect" {
  # ... AWS Direct Connect para OCI
}
