## Spike to test a bug in Zuul

- `cd spring-tomcat-server`
- `./gradlew bootRun`
- `curl --location --request POST 'localhost:8080/upload' \
--form 'file=@"/path/to/large/file"'`

- `cd spring-zuul`
- `./gradlew bootRun`
- `curl --location --request POST 'localhost:8080/upload' \
--form 'file=@"/path/to/large/file"'`

### Behaviour of embedded-tomcat

Embedded tomcat will immediately reject the request based on the header if the `content-length` too large

### Behaviour of Zuul in Spring with Embedded Tomcat

(Zuul (Tomcat))

We've used Zuul with Spring so the embedded tomcat will do the same

Zuul starts to proxy the request, Tomcat rejects it and throws an exception 
because of the file size, however Zuul needs to return a 
response so it hits `/error` in the downstream

The rabbit hole goes deeper

If the requested request in POST then the downstream `/error` is also a POST call

Let's go even deeper in this hole

```java
@ExceptionHandler(MaxUploadSizeExceededException.class)
public String handleFileSizeException(MaxUploadSizeExceededException err) {
  log.error("File is too large", err);
  return "upload-error";
}
```

If I add an `ExceptionControllerAdvice` to handle `MaxUploadSizeExceededException` then 
the downstream error endpoint becomes POST `upload-error`

```java
@ExceptionHandler(MaxUploadSizeExceededException.class)
public String handleFileSizeException(MaxUploadSizeExceededException err) {
  log.error("File is too large", err);
  return "unable to upload error";
}
```

So now Zuul tries to hit the endpoint `/unable to upload error`

```
2021-01-07 22:42:02.755  WARN 39467 --- [nio-4000-exec-1] o.s.c.n.z.f.pre.PreDecorationFilter      : No route found for uri: /unable to upload error
```

