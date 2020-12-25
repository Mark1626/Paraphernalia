package com.mark.hazel.service

import org.springframework.cache.annotation.Cacheable
import org.springframework.stereotype.Service

@Service
class GreetService {

    @Cacheable("greetings")
    String greet(String name) {
        return greetPerson(name)
    }

    private String greetPerson(String name) {
        try {
            Thread.sleep(5000);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new RuntimeException(e);
        }
        return "Hello" + name;
    }
}
