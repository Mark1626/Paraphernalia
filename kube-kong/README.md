## Spike to set kong in k8s

- Create local kind cluster with `sh scripts/create-kind.sh`
- Create the resources `terraform apply`
- Apply the ingress which forwards to kong `kubectl apply -n dev -f ingress.yml`

### Cleanup

- `terraform destroy`
- `sh scripts/delete-kind.sh`

