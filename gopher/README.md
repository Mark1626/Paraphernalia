# An experiment with the gopher protocol

[Home](../README.md){: .button}

> **Status** - Working

## Steps to run the gopher server

**WARNING: Have not set the password for the redis instance since this was an experiment**

```s
docker run -it --rm --name my-running-script -v "$PWD":/usr/src/myapp -w /usr/src/myapp ruby:alpine /bin/sh

gem install redis

./gopher2redis.rb --host host.docker.internal --port 70 \
    --root gopher \
    --localhost host.docker.internal --localport 70

```
