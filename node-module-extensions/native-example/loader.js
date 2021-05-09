const builtin = require("module");
const fs = require("fs");
const yaml = require("yaml");

const Module = module.constructor.length > 1 ? module.constructor : builtin;

const transform = (code) => {
  const patterns = [
    { patt: /\$/g, repl: "this" },
    { patt: /fn \(/g, repl: "function (" },
  ];

  patterns.forEach((pattern) => {
    code = code.replace(pattern.patt, pattern.repl);
  });
  return code;
};

const oldLoader = Module._extensions[".js"];
/**
 * Simplified version of the code from pirates
 * MIT License
 * Copyright (c) 2016-2018 Ari Porad
 * https://github.com/ariporad/pirates/blob/master/LICENSE
 */
Module._extensions[".js"] = function customLoader(mod, filename) {
  let compile = mod._compile;
  mod._compile = function _compile(code) {
    // reset the compile immediately as otherwise we end up having the
    // compile function being changed even though this loader might be reverted
    // Not reverting it here leads to long useless compile chains when doing
    // addHook -> revert -> addHook -> revert -> ...
    // The compile function is also anyway created new when the loader is called a second time.
    mod._compile = compile;
    const newCode = transform(code);

    return mod._compile(newCode, filename);
  };

  oldLoader(mod, filename);
};

/*
 Loader for yaml, based on the json loader in nodejs
 https://github.com/nodejs/node/blob/master/lib/internal/modules/cjs/loader.js#L1143-L1157
*/
function yamlLoader(mod, filename) {
  const content = fs.readFileSync(filename, "utf8");

  try {
    mod.exports = yaml.parse(content)
  } catch (err) {
    err.message = filename + ": " + err.message;
    throw err;
  }
}

Module._extensions[".yaml"] = yamlLoader;
Module._extensions[".yml"] = yamlLoader;
