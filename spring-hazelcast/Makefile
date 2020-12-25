REGISTRY_URL=localhost:5000
IMAGE_NAME=spring-hazelcast
IMAGE_TAG=1.0.0

create-jar:
	./gradlew bootJar

build-image: create-jar
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .

push-image: build-image
	docker tag $(IMAGE_NAME):$(IMAGE_TAG) $(REGISTRY_URL)/$(IMAGE_NAME):$(IMAGE_TAG)
	docker push $(REGISTRY_URL)/$(IMAGE_NAME):$(IMAGE_TAG)

create-kind:
	sh scripts/$@.sh

delete-kind:
	sh scripts/$@.sh

switch-context:
	kubectx kind-kind

ingress-controller: switch-context
	kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/master/deploy/static/provider/kind/deploy.yaml

namespace:
	kubectl create namespace app

rbac:
	kubectl apply -n app -f k8s/$@.yaml

service:
	kubectl apply -n app -f k8s/$@.yaml

deployment:
	kubectl apply -n app -f k8s/$@.yaml

config:
	kubectl apply -n app -f k8s/$@.yaml

ingress:
	kubectl apply -n app -f k8s/$@.yaml

deploy-all: switch-context rbac service config deployment ingress

clean-all:
	kubectl delete clusterrolebinding.rbac.authorization.k8s.io spring-hazelcast-rbac
	kubectl delete service -n app spring-hazelcast-service
	kubectl delete configmap -n app spring-hazelcast-configuration
	kubectl delete statefulset -n app spring-hazelcast-deployment
	kubectl delete ingress -n app spring-hazelcast-ingress

.PHONY: create-jar build-image push-image \
create-kind delete-kind ingress-controller \
switch-context deploy-all clean-all \
namespace switch-context rbac service config deployment ingress
