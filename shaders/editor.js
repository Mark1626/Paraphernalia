import errorParser from 'gl-shader-errors'
import triangle from 'a-big-triangle'
import CodeMirror from 'codemirror';
import Shader from 'gl-shader'

import './editor-mode-glsl'

export default () => {
  const container = document.querySelector('.editor')
  const defaultFragmentShader = `
void main() {
  gl_FragColor = vec4(1, 0.6, 0.5, 1);
}
  `
  var vert = `
    precision mediump float;

    attribute vec2 position;
    varying vec2 uv;

    void main() {
      uv = position * 0.5 + 0.5;
      gl_Position = vec4(position, 1, 1);
    }
  `

  var cOpts = { preserveDrawingBuffer: true }
  var canvasDiv = document.querySelector('#canvas')
  var shaderCanvas = canvasDiv.appendChild(document.createElement('canvas'))
  var gls = shaderCanvas.getContext('webgl', cOpts) || shaderCanvas.getContext('experimental-webgl', cOpts)
  gls.getExtension('OES_standard_derivatives')
  var shader = Shader(gls, vert, getFrag(defaultFragmentShader))

  var currTime = 0
  var start = Date.now()
  var shape = []

  // var rem = 16

  // TODO: Fix canvas resize
  function resize (e) {
    // var height = window.innerHeight - 5 * rem - 2
    // const canvas = document.getElementById("canvas")
    canvasDiv.style.minWidth = '500px'
    shaderCanvas.style.height = "100%"
    shaderCanvas.style.width = "100%"
  }

  function loop () {
    currTime = (Date.now() - start) / 100
    window.requestAnimationFrame(loop)

    draw(gls, shader)
  }

  // omg loads of work to compare buffers
  // var matchLabel = document.querySelector('.mainui .checker span')
  // var pixelBuffer1 = new Uint8Array(4 * 512 * 512)
  // var pixelBuffer2 = new Uint8Array(4 * 512 * 512)
  // var failedMatch = false
  var prefixLineCount = 2

  function draw (gl, shader) {
    var width = gl.canvas.width
    var height = gl.canvas.height
    gl.viewport(0, 0, width, height)
    shape[0] = width
    shape[1] = height
    shader.bind()
    shader.uniforms.iResolution = shape
    shader.uniforms.iGlobalTime = currTime

    gl.clearColor(0, 0, 0, 1)
    gl.clear(gl.COLOR_BUFFER_BIT)
    triangle(gl)
  }

  var editor = new CodeMirror(container, {
    value: defaultFragmentShader,
    theme: 'xq-light',
    viewportMargin: Infinity,
    lineNumbers: true,
    gutters: [
      'shaderError',
      'CodeMirror-linenumbers'
    ]
  })

  var noop = function(){}
  editor.on('change', function () {
    var frag = getFrag(editor.getValue())
    var warn = console.warn

    editor.clearGutter('shaderError')

    try {
      console.warn = noop
      shader.update(vert, frag)
      console.warn = warn
    } catch (e) {
      var errors = errorParser(e.rawError)
      for (var i = 0; i < errors.length; i++) {
        var err = errors[i]
        var line = err.line - prefixLineCount
        var el = document.createElement('div')
        el.style.width = '8px'
        el.style.height = '8px'
        el.style.borderRadius = '8px'
        el.style.background = '#f00'
        el.style.marginTop = '6px'
        el.title = errors[i].message
        editor.setGutterMarker(line - 1, 'shaderError', el)
      }
      return
    }
  })

  function getFrag (src) {
    return '#extension GL_OES_standard_derivatives : enable\nprecision highp float;\n' + src
  }

  resize()
  window.addEventListener('resize', resize)
  window.requestAnimationFrame(loop)
}
