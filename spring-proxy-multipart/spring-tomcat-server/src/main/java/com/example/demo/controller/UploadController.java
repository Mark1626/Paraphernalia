package com.example.demo.controller;

import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.multipart.MultipartFile;

@RestController
public class UploadController {
    @PostMapping("/upload")
    public ResponseEntity<String> uploadFile(@RequestParam("document") MultipartFile file) {
        return ResponseEntity.ok("OK");
    }

    @PostMapping("/upload-error")
    public ResponseEntity<String> uploadErrorEndpoint() {
        return ResponseEntity.ok("Upload Error");
    }

    @PostMapping("/error")
    public ResponseEntity<String> errorEndpoint() {
        return ResponseEntity.ok("Error endpoint");
    }
}
