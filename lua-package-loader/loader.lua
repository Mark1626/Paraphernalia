-- This will transform some patterns in our file
local function transform(s)
  local str = s
  local int = "([%d]+)"
  
  local patterns = {
    { patt = "@", repl = "self" },
    { patt = "&&", repl = " and " },
    { patt = "||", repl = " or " },
    { patt = "fn%(", repl = "function(" },
  }

  for _, v in ipairs(patterns) do str = str:gsub(v.patt, v.repl) end
  return str
end

-- Override the default loader
table.insert(package.loaders, 2, function(modulename)
  local modulepath = string.gsub(modulename, "%.", "/")
  for path in string.gmatch(package.path, "([^;]+)") do
    local filename = string.gsub(path, "%?", modulepath)
    local file = io.open(filename, "rb")
    if file then
      local content = assert(file:read("*a"))
      local transformed_file = transform(content)
      return assert(loadstring(transformed_file, modulename))
    end
  end
  return "Unable to load file " .. modulename
end)
