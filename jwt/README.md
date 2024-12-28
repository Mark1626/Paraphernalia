# JWT PoC

Simple app with basic JWT auth

1. Get a JWT token through `curl http://127.0.0.1:4567/token`

2. `curl --header "Authorization: $JWT_TOKEN" http://127.0.0.1:4567/api/hello`
