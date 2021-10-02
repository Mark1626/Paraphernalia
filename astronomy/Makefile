.ONESHELL:

download_sao:
	mkdir -p sao
	cd sao
	curl -O http://tdc-www.harvard.edu/software/catalogs/saopc.tar.gz
	tar -xvf saopc.tar.gz
	cd ..

download_catalog: download_sao

.PHONY: download_catalog download_sao
