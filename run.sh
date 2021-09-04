#!/bin/bash
image=amulator
docker build -t $image .
docker run --rm -v `pwd`:/home -v `pwd`/resources/:/setup/ -it $image bash