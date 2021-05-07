const gifFrames = require('gif-frames');
const fs = require('fs');
const path = require('path')

gifFrames(
  { url: path.resolve('bird.gif'), frames: 'all', outputType: 'png', cumulative: true },
  function (err, frameData) {
    if (err) {
      throw err;
    }
    frameData.forEach(function (frame) {
      frame.getImage().pipe(fs.createWriteStream(
        'image-' + frame.frameIndex + '.png'
      ));
    });
  }
);