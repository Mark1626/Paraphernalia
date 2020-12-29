## Spike to test localMount in kind

- Create cluster `make create-kind`
- Create the namespace `make namespace`
- Export `DB_PASSWORD` and `make secrets`
- Run `make postgres` to deploy the postgres instance within the cluster

### Usage

- `kubectl port-forward -n app pods/postgres-pod 5432:5432`

You should be able to login with `psql`

#### Cleanup

- `make delete-kind`
