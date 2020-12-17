FROM openjdk:11

RUN apt-get update && \
  apt-get install -y graphviz && \
  wget -O /home/plantuml.jar http://sourceforge.net/projects/plantuml/files/plantuml.jar/download

WORKDIR /home/seq

ENTRYPOINT ["java", "-jar", "/home/plantuml.jar"]
