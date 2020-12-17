FILES = $(wildcard *.lua)

upload-all:
	for file in $(FILES); do \
		nodemcu-tool upload $$file; \
	done

.PHONY: upload-all
