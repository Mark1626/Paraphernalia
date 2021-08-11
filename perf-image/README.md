# Perf image

Docker image which contains perf, valgrind for working on performance tuning

## Building image

`docker build -t perf-image`

## Running the image

`docker run --rm -it -v $PWD:/home/mark1626 --security-opt seccomp=seccomp-perf.json perf-image:version`
