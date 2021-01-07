package com.example.demo;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.ControllerAdvice;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.multipart.MaxUploadSizeExceededException;

@ControllerAdvice
public class ExceptionControllerAdvice {

    Logger log = LoggerFactory.getLogger(ExceptionControllerAdvice.class);

//    @ExceptionHandler(MaxUploadSizeExceededException.class)
//    public String handleFileSizeException(MaxUploadSizeExceededException err) {
//        log.error("File is too large", err);
//        return "unable to upload error";
//    }
}
