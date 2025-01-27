terraform {
  backend "s3" {
    bucket = "hydra-doom-tf"
    key    = "clusters/hydra-doom-dev-cluster/tfstate.global"
    region = "us-east-1"
  }
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.22.0"
    }
  }
}

variable "eks_cluster_arn" {
  type        = string
  description = "The ARN of the EKS cluster."
}

variable "ssl_cert_arn" {
  type = string
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = var.eks_cluster_arn
}

module "global" {
  source = "../../bootstrap/global/"

  ssl_cert_arn    = var.ssl_cert_arn
  external_domain = "us-east-1.hydra-doom.sundae.fi"
  thanos_endpoints = [
    "k8s-hydradoo-thanossi-3e6cc6bace-ddd76e7d5e148d9f.elb.us-east-1.amazonaws.com:10901",
    "k8s-hydradoo-thanossi-08d03cf670-c832566453f2a5a0.elb.eu-central-1.amazonaws.com:10901",
    "k8s-hydradoo-thanossi-ee478628bf-6cd33f983e73ae8f.elb.us-west-2.amazonaws.com:10901",
    "k8s-hydradoo-thanossi-42bd8b32e4-0818e533e76d43c3.elb.ap-southeast-1.amazonaws.com:10901",
    "k8s-hydradoo-thanossi-e1183b7f93-f951f656c1c3da07.elb.sa-east-1.amazonaws.com:10901",
    "k8s-hydradoo-thanossi-13ead426cd-5339600f2c046547.elb.af-south-1.amazonaws.com:10901",
  ]
}
