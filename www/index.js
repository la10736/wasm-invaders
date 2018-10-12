import { Game, SpaceInvaders } from "wasm-invaders";
import { memory } from "wasm-invaders/wasm_invaders_bg";


const game = Game.new();
const si = game.space_invaders();
const width = game.width();
const height = game.height();

const canvas = document.getElementById("screen");
const inMemoryCanvas = document.createElement('canvas');
const inMemoryCanvasCtx = inMemoryCanvas.getContext('2d');
const w = width * 3;
const h = height * 3;
inMemoryCanvas.width = width;
inMemoryCanvas.height = height;
canvas.width = h;
canvas.height = h;

const ctx = canvas.getContext('2d');
const heading = document.getElementById("heading");
heading.textContent = game.name();

const playPauseBtn = document.getElementById("play-pause");
const coinBtn = document.getElementById("coin");
const playBtn = document.getElementById("play");

const imgData = new ImageData(width, height);

let animationId = null;

const render = () => {

    si.next_frame();

    fps.render()

    draw();
};

const renderLoop = () => {
    requestAnimationFrame(render);

    animationId = setTimeout(renderLoop, 1000 / 60);
};

const BLACK = [0, 0, 0];
const WHITE = [255, 255, 255];
const RED = [255, 0, 0];
const GREEN = [0, 255, 0];

const color = (row, col) => {
    col = 256 - col;
    if (col <= 32) {
        return WHITE;
    }
    if (col <= 64) {
        return RED;
    }
    if (col <= 184) {
        return WHITE;
    }
    if (col <= 240) {
        return GREEN;
    }
    if (row > 16 && row <= 134) {
        return GREEN;
    }
    return WHITE
}

const draw = () => {
    const vramPtr = game.vram();
    const vram = new Uint8Array(memory.buffer, vramPtr, (width * height / 8 ));

    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col);
            let p = idx>>3;
            let val = (vram[p] & (0x1 << ( idx & 0x7 )))

            let c = val === 0 ? BLACK : color(row, col);

            let pos = ((row * width) + col) * 4;
            imgData.data[pos] = c[0];
            imgData.data[pos+1] = c[1];
            imgData.data[pos+2] = c[2];
            imgData.data[pos+3] = 255;
        }
    }
    inMemoryCanvasCtx.putImageData(imgData,0,0);
    ctx.imageSmoothingEnabled = false;
    ctx.save();
    ctx.translate(w/2, h/2);
    ctx.rotate( 3 * Math.PI / 2 );
    ctx.translate(-h/2, -w/2);
    ctx.drawImage(inMemoryCanvas, 0, 0, w, h);
    ctx.restore();
};

const getIndex = (row, col) => {
  return row * width + col;
};

const play = () => {
    playPauseBtn.textContent = "⏸";
    renderLoop();
}

const pause = () => {
    playPauseBtn.textContent = "▶";
    // cancelAnimationFrame(animationId);
    clearTimeout(animationId);
    animationId = null;
}

playPauseBtn.addEventListener("click", event => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});

const isPaused = () => {
    return animationId === null;
}

const addGameButton = (btn, act) => {
    btn.addEventListener("mousedown", event => {
        act(true);
    });

    btn.addEventListener("mouseup", event => {
        act(false);
    });
}

const coin = (v) => { si.coin(v) };
const plr = (v) => { si.play(v) };

addGameButton(coinBtn, coin);
addGameButton(playBtn, plr);

const keyboard = (event) => {
    const pressed = event.type === "keydown";
    switch (event.key) {
        case "ArrowLeft":
            // Left pressed
            si.left(pressed);
            break;
        case "ArrowRight":
            // Right pressed
            si.right(pressed);
            break;
        case " ":
            // Space pressed
            si.shoot(pressed);
            break;
        default: return;
    }
    event.preventDefault();
}

document.addEventListener("keydown", event => {
    keyboard(event);
});

document.addEventListener("keyup", event => {
    keyboard(event);
});

const fps = new class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = 1 / delta * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    // Render the statistics.
    this.fps.textContent = `
Frames per Second:
         latest = ${Math.round(fps)}
avg of last 100 = ${Math.round(mean)}
min of last 100 = ${Math.round(min)}
max of last 100 = ${Math.round(max)}
`.trim();
  }
};

play();
