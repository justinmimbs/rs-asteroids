import init, { PathList } from './wasm/polygon.js';

const canvas = document.querySelector('#canvas')
const context = canvas.getContext('2d');

canvas.width = 400;
canvas.height = 400;
context.strokeStyle = '#666666';
context.lineWidth = 1;
context.lineCap = 'round';
context.lineJoin = 'round';

function drawPolygon(s, tx, ty, length, points) {
    context.beginPath();
    context.moveTo(points[0] * s + tx, points[1] * s + ty);
    for (let i = 1; i < length; i += 1) {
        context.lineTo(points[i * 2] * s + tx, points[i * 2 + 1] * s + ty);
    }
    context.closePath();
    context.stroke();
}

async function main() {
    const { memory } = await init();
    const list = PathList.new();
    const listLength = list.length();
    const paths = new Uint32Array(memory.buffer, list.paths(), listLength * 2);
    for (let i = 0; i < listLength; i += 1) {
        let length = paths[i * 2];
        let points = new Float64Array(memory.buffer, paths[i * 2 + 1], length * 2);
        drawPolygon(40, 50 + i * 100, 100, length, points);
    }
    list.free();
}

main();
