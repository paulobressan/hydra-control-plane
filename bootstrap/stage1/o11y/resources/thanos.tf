resource "kubernetes_service_v1" "thanos_sidecar" {
  metadata {
    name      = "thanos-sidecar"
    namespace = var.namespace
    annotations = {
      "service.beta.kubernetes.io/aws-load-balancer-nlb-target-type" : "instance"
      "service.beta.kubernetes.io/aws-load-balancer-scheme" : "internet-facing"
      "service.beta.kubernetes.io/aws-load-balancer-type" : "external"
    }
  }

  spec {
    load_balancer_class = "service.k8s.aws/nlb"
    selector = {
      "prometheus"                  = "prometheus"
      "app.kubernetes.io/instance"  = "prometheus"
      "operator.prometheus.io/name" = "prometheus"
    }

    port {
      name        = "web"
      port        = 9090
      target_port = "web"
    }

    port {
      name        = "grpc"
      port        = 10901
      target_port = 10901
    }

    type = "LoadBalancer"
  }
}
