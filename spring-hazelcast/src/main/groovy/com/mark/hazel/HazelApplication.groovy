package com.mark.hazel

import org.springframework.boot.SpringApplication
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.cache.annotation.EnableCaching

@EnableCaching
@SpringBootApplication
class HazelApplication {

	static void main(String[] args) {
		SpringApplication.run(HazelApplication, args)
	}

}
