#!/bin/bash

temp=$(vcgencmd measure_temp | grep -o '[0-9]*\.[0-9]*')
timestamp=$(date +'%s')
str=$(printf "time=%-15stemp=%5s\n" "$timestamp" "$temp")
echo $str >> /home/pi/telemetry/temperature.log
