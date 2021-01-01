print "Starting Application"
dofile("redis.lua")

-- Gpio Pins
pin = 4
gpio.mode(pin, gpio.OUTPUT)
gpio.write(pin, gpio.LOW)
on = 0

-- Redis Connection Info
host = "<host-ip>"
port = "6379"
local redis = dofile("redis.lua").connect(host, port)

-- Timer
mytimer = tmr.create()
mytimer:register(5000, tmr.ALARM_AUTO, function()
  if on == 0 then 
    on = 1 
    gpio.write(pin, gpio.HIGH)
  else 
    on = 0
    gpio.write(pin, gpio.LOW)
  end 
end)

print("Listening redis channel for updates")
redis:subscribe("lamp", function(channel, msg) 
  if string.find(msg, "blink")
  then
    print("Starting blink on pin" .. pin)
    mytimer:start()
  elseif string.find(msg, "start")
  then
    print("Setting gpio pin" .. pin .. " to high")
    gpio.write(pin, gpio.HIGH)
    mytimer:stop()
  elseif string.find(msg, "stop")
  then
    print("Setting gpio pin" .. pin .. " to low")
    gpio.write(pin, gpio.LOW)
    mytimer:stop()
  else
    print("Did not match any command " .. msg)
  end
  print(channel, msg)
end)
