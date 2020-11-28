FROM busybox:latest

RUN adduser -D -h /home/web web && \
  chown -R web:web /home/web

USER web

WORKDIR /home/web

ENV HTTP_PORT 8080
ENV HTTP_CONF /home/web/http.conf
ENV HTTP_WEB_DIR /home/web/www

ENTRYPOINT httpd -f -p $HTTP_PORT -c $HTTP_CONF -h $HTTP_WEB_DIR
