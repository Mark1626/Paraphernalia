#!/bin/sh
set -o errexit

# create a cluster with the local registry enabled in containerd with ingress
cat <<EOF | kind create cluster --config=-
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  extraMounts:
  - hostPath: $PWD/volume
    containerPath: /files
EOF
