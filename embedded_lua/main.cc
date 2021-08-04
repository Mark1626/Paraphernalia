#include <iostream>
#include <string>
#include <lua/lua.hpp>

int api_add(lua_State *l) {
  int a = luaL_checkinteger(l, 1);
  int b = luaL_checkinteger(l, 2);

  lua_pushnumber(l, a+b);

  return 1;
}

int get_integer(lua_State *l, std::string field) {
  lua_getglobal(l, field.c_str());
  const int val = lua_tointeger(l, -1);
  lua_pop(l, 1);
  return val;
}

class LuaEmbedded {
  lua_State *l;

public:
  LuaEmbedded() {
    l = luaL_newstate();
    luaL_openlibs(l);
    setup_api();
  }

  ~LuaEmbedded() { lua_close(l); }


  void setup_api() {
    lua_pushcfunction(l, api_add);
    lua_setglobal(l, "add");
  }

  void execute_script(std::string filename) {
    if (luaL_loadfile(l, filename.c_str()) == LUA_OK) {
      if (lua_pcall(l, 0, 1, 0) == LUA_OK) {
        lua_pop(l, lua_gettop(l));
      } else {
        std::cout << "Error Running file\n";
      }
    } else {
      std::cout << "Error opening file\n";
    }
  }
};

int main() {
  LuaEmbedded a;
  a.execute_script("test.lua");
}
