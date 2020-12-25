package com.mark.hazel.controller

import com.mark.hazel.service.GreetService
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.RequestParam
import org.springframework.web.bind.annotation.RestController

@RestController
class PingController {

    private final GreetService greetService;

    PingController(GreetService greetService) {
        this.greetService = greetService;
    }

    @GetMapping("/greet")
    ResponseEntity<String> sayPong(@RequestParam("name") String name) {
        return ResponseEntity.ok(greetService.greet(name));
    }
}
