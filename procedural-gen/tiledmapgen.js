const { Jimp } = require("jimp");
const { SimpleTiledModel } = require("wavefunctioncollapse");

const normalizeSeed = (seed) => {
    if (typeof seed === 'number') {
        seed = Math.abs(seed);
    } else if (typeof seed === 'string') {
        const string = seed;
        seed = 0;

        for (let i = 0; i < string.length; i++) {
            seed = (seed + (i + 1) * (string.charCodeAt(i) % 96)) % 2147483647;
        }
    }

    if (seed === 0) {
        seed = 311;
    }

    return seed;
}

const lcgRandom = (seed) => {
    let state = normalizeSeed(seed);

    return () => {
        const result = (state * 48271) % 2147483647;
        state = result;
        return result / 2147483647;
    };
}

const loadTile = async (basePath, tile, number) => {
    const unique = number !== null;
    const tilePath = basePath + tile.name + (unique ? "_" + number : "") + ".png";
    const image = await Jimp.read(tilePath);

    if (unique) {
        tile.bitmap[number] = new Uint8Array(image.bitmap.data);
    } else {
        tile.bitmap = new Uint8Array(image.bitmap.data);
    }
}

const loadTilemap = async (definition) => {
    const promises = []
    const path = definition.path
    const unique = !!definition.unique

    definition.tiles.map((tile) => {
        if (unique) {
            if (tile.symmetry == "X") {
                tile.bitmap = new Array(1)
                promises.push(loadTile(path, tile, 0));
            } else {
                tile.bitmap = new Array(4)
                promises.push(loadTile(path, tile, 0));
                promises.push(loadTile(path, tile, 1));
                promises.push(loadTile(path, tile, 2));
                promises.push(loadTile(path, tile, 3));
            }
        } else {
            promises.push(loadTile(path, tile, null));
        }
    })

    return Promise.all(promises);
}

const generateMap = async (definition, destWidth, destHeight, destImagePath) => {
    await loadTilemap(definition);
    const model = new SimpleTiledModel(definition, null, destWidth, destHeight, false);

    const finished = model.generate(lcgRandom("Test"));
    if (finished) {
        const result = model.graphics();
        const outputImage = Jimp.fromBitmap({ data: result, width: destWidth * definition.tilesize, height: destHeight * definition.tilesize });
        return outputImage.write(destImagePath);
    }

    throw new Error("Failed to converge");
}

generateMap(require("./monochrome"), 32, 32, "monochrome.png");
