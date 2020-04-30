.PHONY: build start

build:
	env GOOS=linux GOARCH=arm go build
	chmod +x static-server

start:
	./static-server
