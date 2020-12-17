print "Starting Application"
dofile("httpServer.lua")

-- Gpio Pins
pin = 4
gpio.mode(pin, gpio.OUTPUT)
gpio.write(pin, gpio.LOW)
on = 0

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

-- Service Discovery
cron.schedule("* * * * *", function(e)
  ip = wifi.sta.getip()
  print('Registering in Service Discovery')
  http.get("<service>/register?ip=" .. ip, nil, function(code, data)
    if (code < 0) then
      print("HTTP request failed")
    else
      print("HTTP request passed" .. code)
    end
  end)
end)

-- Http Server
httpServer:listen(80)

httpServer:use('/blink', function(req, res)
  print('Blink received request')
  mytimer:start()
	res:send('Blinking')
end)

httpServer:use('/off', function(req, res)
  print('Stop Request Received')
  gpio.write(pin, gpio.LOW)
  mytimer:stop()
	res:send('Stopping')
end)
