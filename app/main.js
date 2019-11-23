import init, { example } from './wasm/polygon.js';

const canvas = document.querySelector('#canvas')
const context = canvas.getContext('2d');

canvas.width = 400;
canvas.height = 400;
context.strokeStyle = '#666666';
context.lineWidth = 1;
context.lineCap = 'round';
context.lineJoin = 'round';

function drawPoints(points, offset, length, alpha, isClosed) {
    context.beginPath();
    context.globalAlpha = alpha;
    context.moveTo(points[offset * 2], points[offset * 2 + 1]);
    for (let i = offset + 1; i < offset + length; i += 1) {
        context.lineTo(points[i * 2], points[i * 2 + 1]);
    }
    if (isClosed) {
        context.closePath();
    }
    context.stroke();
}

async function main() {
    const { memory } = await init();
    const list = example();

    // render
    const length = list.length();
    const paths = new Uint32Array(memory.buffer, list.paths(), length * 2);
    const alphas = new Float64Array(memory.buffer, list.alphas(), length);
    const ends = new Uint8Array(memory.buffer, list.ends(), length);
    const points = new Float64Array(memory.buffer, list.points(), list.points_length() * 2);

    for (let i = 0; i < length; i += 1) {
        drawPoints(points, paths[i * 2], paths[i * 2 + 1], alphas[i], ends[i] === 1);
    }
    list.free();
}

main();
