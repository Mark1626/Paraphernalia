create-kind:
	sh scripts/$@.sh

delete-kind:
	sh scripts/$@.sh

switch-context:
	kubectx kind-kind

namespace: switch-context
	kubectl create namespace app

secrets: switch-context
	kubectl create -n app secret generic app-secret --from-literal=DB_PASSWORD=$$DB_PASSWORD

postgres: switch-context
	kubectl apply -n app -f $@.yml

.PHONY: create-kind delete-kind switch-context \
namespace secrets postgres
