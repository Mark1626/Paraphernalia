require "loader"
require "cube"

local c = Cube:new({ a = 4 })
print("Perimeter of cube " .. c:perimeter())
print("Area of cube " .. c:area())

require 'logical'
