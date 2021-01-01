provider "helm" {
  version = "~>1.3"
}

resource "helm_release" "kong" {
  name       = "kong-ingress"
  namespace  = "dev"
  repository = "https://charts.konghq.com"
  chart      = "kong"
  version    = "1.9.0"

  set {
    name  = "ingressController.installCRDs"
    value = "false"
  }

  set {
    name  = "ingressController.ingressClass"
    value = "kong-ingress"
  }

  set {
    name  = "proxy.type"
    value = "ClusterIP"
  }

  set {
    name  = "env.headers"
    value = "off"
  }

  set {
    name  = "ingressController.installCRDs"
    value = "false"
  }
}

resource "helm_release" "hello-world" {
  name       = "hello-world"
  namespace  = "dev"
  chart = "./hello-world"
}
