package com.example.demo.controller;

import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.multipart.MultipartFile;

@RestController
public class UploadController {
    @PostMapping("/upload-proxy")
    public ResponseEntity<String> uploadData(@RequestParam("document") MultipartFile file) {
        return ResponseEntity.ok("OK");
    }
}
