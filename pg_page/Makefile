start-pg:
	docker run --name pg_instance \
	-v $$PWD/data:/var/lib/postgresql/data \
	-p 5432:5432 \
	-e POSTGRES_USER=mark \
	-e POSTGRES_PASSWORD=abc \
	-d postgres

stop-pg:
	docker stop pg_instance

delete-pg:
	docker rm pg_instance
