// The browser half of the mini project. NOT run while writing the lesson:
// build ../pkg/ first with `wasm-pack build --target web` (see index.html).
//
// `--target web` makes pkg/webassembly.js an ES module whose default export,
// init(), fetches webassembly_bg.wasm and starts it (running the Rust
// `#[wasm_bindgen(start)]` function). The named exports are what the Rust
// side marked with #[wasm_bindgen]: the `Universe` class and `patternNames`.
import init, { Universe, patternNames } from "../pkg/webassembly.js";

const WIDTH = 64;
const HEIGHT = 48;
const CELL = 10; // CSS pixels per cell

// init() resolves to the module's exports, including its linear `memory`.
const wasm = await init();

let universe = new Universe(WIDTH, HEIGHT);
universe.insertPattern("glider", 2, 2);
universe.insertPattern("pulsar", 16, 24);

const canvas = document.getElementById("board");
const context = canvas.getContext("2d");
const colours = getComputedStyle(document.documentElement);

// Draw at the screen's real pixel density, so cells stay sharp.
function sizeCanvas() {
  const ratio = window.devicePixelRatio || 1;
  canvas.style.width = `${universe.width * CELL}px`;
  canvas.style.height = `${universe.height * CELL}px`;
  canvas.width = universe.width * CELL * ratio;
  canvas.height = universe.height * CELL * ratio;
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
}

function draw() {
  // Zero-copy: a view straight into Rust's Vec<Cell>, one byte per cell.
  // Make a NEW view every frame: if the WebAssembly memory ever grows, the
  // old memory.buffer is detached and an old view would read nothing.
  const cells = new Uint8Array(
    wasm.memory.buffer,
    universe.cellsPtr(),
    universe.width * universe.height,
  );
  const alive = colours.getPropertyValue("--alive").trim();
  const dead = colours.getPropertyValue("--dead").trim();
  for (let row = 0; row < universe.height; row++) {
    for (let col = 0; col < universe.width; col++) {
      context.fillStyle = cells[row * universe.width + col] === 1 ? alive : dead;
      context.fillRect(col * CELL, row * CELL, CELL - 1, CELL - 1);
    }
  }
  document.getElementById("stats").value =
    `generation ${universe.generation}, ${universe.population} alive`;
}

// ---- Animation ---------------------------------------------------------------------
let playing = false;
const playButton = document.getElementById("play");

function frame() {
  if (!playing) return;
  universe.tick();
  draw();
  requestAnimationFrame(frame);
}

playButton.addEventListener("click", () => {
  playing = !playing;
  playButton.textContent = playing ? "Pause" : "Play";
  if (playing) requestAnimationFrame(frame);
});

document.getElementById("step").addEventListener("click", () => {
  universe.tick();
  draw();
});

document.getElementById("random").addEventListener("click", () => {
  universe.randomize(0.3); // Rust calls Math.random() through the JsHost
  draw();
});

document.getElementById("clear").addEventListener("click", () => {
  universe.clear();
  draw();
});

// ---- Clicking a cell -----------------------------------------------------------------
canvas.addEventListener("click", (event) => {
  const box = canvas.getBoundingClientRect();
  const col = Math.floor(((event.clientX - box.left) / box.width) * universe.width);
  const row = Math.floor(((event.clientY - box.top) / box.height) * universe.height);
  universe.toggle(row, col);
  draw();
});

// ---- Built-in patterns -------------------------------------------------------------------
const select = document.getElementById("pattern");
for (const name of patternNames()) {
  select.add(new Option(name, name));
}
document.getElementById("insert").addEventListener("click", () => {
  universe.insertPattern(select.value, 1, 1);
  draw();
});

// ---- A pattern typed by the user -------------------------------------------------------------
// fromPattern returns Result<Universe, JsError> in Rust: an Err arrives here
// as a thrown JavaScript Error, with the Rust error's message.
document.getElementById("load").addEventListener("click", () => {
  const error = document.getElementById("error");
  try {
    const next = Universe.fromPattern(document.getElementById("custom").value, WIDTH, HEIGHT);
    universe.free(); // the old Rust value is NOT garbage-collected: free it
    universe = next;
    error.textContent = "";
    sizeCanvas();
    draw();
  } catch (e) {
    error.textContent = e.message; // e.g. "line 3, column 2: 'X' is not a cell ..."
  }
});

sizeCanvas();
draw();
