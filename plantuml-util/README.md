# Plant-UML Docker

A utility docker image to generate plant-uml diagrams

### Build Image

`docker build -t plant-uml:v1 .`

### Creating an 

`docker run --rm -it -v $PWD:/home/seq plant-uml:v1 sequence.txt`

#### TODO

- Add user inside docker image
- Multistage docker image
