package com.mark.config;

import io.jaegertracing.internal.JaegerTracer;
import io.jaegertracing.internal.reporters.RemoteReporter;
import io.jaegertracing.internal.samplers.ProbabilisticSampler;
import io.jaegertracing.thrift.internal.senders.HttpSender.Builder;
import io.opentracing.Tracer;
import io.opentracing.contrib.web.servlet.filter.ServletFilterSpanDecorator;
import io.opentracing.noop.NoopTracerFactory;
import io.opentracing.util.GlobalTracer;
import java.util.Arrays;
import java.util.List;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TracingConfig {

    @Value("${spring.application.name}")
    private String applicationName;

    @Value("${tracing.jaeger.endpoint:http://localhost:14268/api/traces}")
    private String jaegerEndpoint;

    @Value("${tracing.enabled:false}")
    private boolean tracingEnabled;

    @Bean
    public Tracer tracer() {
        if (!tracingEnabled) {
            return NoopTracerFactory.create();
        }

        RemoteReporter reporter = new RemoteReporter.Builder()
            .withSender(new Builder(jaegerEndpoint).build())
            .build();
        JaegerTracer tracer = new JaegerTracer.Builder(applicationName)
            .withReporter(reporter)
            .withSampler(new ProbabilisticSampler(1.0))
            .build();
        GlobalTracer.registerIfAbsent(tracer);
        return tracer;
    }

    @Bean
    public List<ServletFilterSpanDecorator> spanDecorators() {
        return Arrays.asList(
            ServletFilterSpanDecorator.STANDARD_TAGS
        );
    }
}
