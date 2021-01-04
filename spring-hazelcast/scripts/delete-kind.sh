#!/bin/sh
set -o errexit

# create registry container unless it already exists
reg_name='kind-registry'
reg_port='5000'
running="$(docker inspect -f '{{.State.Running}}' "${reg_name}" 2>/dev/null || true)"
if [ "${running}" == 'true' ]; then
  docker stop "${reg_name}"
  docker rm "${reg_name}"
fi

kind delete cluster
