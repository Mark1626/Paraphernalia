# Spike for running a http server with CGI

# Running the docker image

```sh
docker run --rm -it -p 8080:8080 -v $PWD/web/www:/home/web/www -v $PWD/web/http.conf:/home/web/http.conf  busybox-http:v1
```
