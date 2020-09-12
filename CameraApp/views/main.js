const html = require('choo/html')
const jsQR = require("jsqr")

const TITLE = 'CameraApp - main'

module.exports = view

// const qrVal = jsQR('./awesome-qr-1.png', 1000, 1000)

function view (state, emit) {
  if (state.title !== TITLE) emit(state.events.DOMTITLECHANGE, TITLE)

  const scanQR = (e) => {
    const file = e.target.files[0];
    console.log("Here")
    if (file) {
      var reader = new FileReader();
      reader.readAsDataURL(file);

      reader.onload = (evt) => {
        const img = new Image()
        img.onload = () => {
          const canvas = document.createElement("canvas")
          // canvas.setAttribute("width", img.width)
          // canvas.setAttribute("height", img.height)
          const ctx = canvas.getContext("2d")
          ctx.drawImage(img, 0, 0)
          const imageData = ctx.getImageData(0, 0, img.width, img.height)
          const val = jsQR(imageData.data, imageData.width, imageData.height)
          console.log("Read Value", val)
        }
        img.src = evt.target.result
      };
      reader.onerror = () => {
        // eslint-disable-next-line no-console
        console.log('error reading file');
      };
    }
  }

  return html`
    <body class="code lh-copy">
      <canvas id="c"></canvas>
      <input type="file" accept="image/*;capture=camera" onchange=${scanQR}>
    </body>
  `
}
