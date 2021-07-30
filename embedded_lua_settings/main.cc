#include <iostream>
#include <lua/lua.hpp>
#include <string>

struct Config {
  std::string name;
  int width;
  int height;
};

const std::string config_name = "settings.lua";

void log_error(const std::string *fmt) { std::cerr << fmt; }

class LuaEmbedder {
  lua_State *l;

public:
  LuaEmbedder() {
    l = luaL_newstate();
    luaL_openlibs(l);
  }

  std::string get_string(std::string field) {
    lua_getglobal(l, field.c_str());
    const char *val = lua_tostring(l, -1);
    lua_pop(l, 1);
    return std::string(val);
  }

  int get_integer(std::string field) {
    lua_getglobal(l, field.c_str());
    const int val = lua_tointeger(l, -1);
    lua_pop(l, 1);
    return val;
  }

  Config load_configuration() {
    if (luaL_loadfile(l, config_name.c_str()) == 0) {
      if (lua_pcall(l, 0, 1, 0)) {
        lua_pop(l, lua_gettop(l));
      }
    }
    std::string name = get_string("name");
    int width = get_integer("width");
    int height = get_integer("height");
    return {.name = name, .width = width, .height = height};
  }

  ~LuaEmbedder() { lua_close(l); }
};

int main() {
  LuaEmbedder a;
  Config config = a.load_configuration();

  std::cout << config.name << "\n";
  std::cout << config.width << " " << config.height << "\n";
}
