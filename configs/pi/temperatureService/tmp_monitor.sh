#!/bin/bash

while true
do
  temp=$(vcgencmd measure_temp | grep -o '[0-9]*\.[0-9]*')
  timestamp=$(date +'%s')
  str=$(printf "time=%-15stemp=%5s\n" "$timestamp" "$temp")
  echo $str >> temperature.log
  sleep 10
done