FROM alpine:latest

RUN apk add --no-cache sqlite
  adduser -D -h /home/orch orch && \
  chown -R orch:orch /home/orch && \
  mkdir -p /var/spool/cron/crontabs

USER orch

WORKDIR /home/orch

ENTRYPOINT ["crond"]
