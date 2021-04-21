Cube = {a = 1}

function Cube:new(o)
  o = o or {}
  setmetatable(o, @)
  @.__index = @
  return o
end

function Cube:perimeter()
  return 4 * @.a
end

function Cube:area()
  return @.a * @.a
end

return Cube
