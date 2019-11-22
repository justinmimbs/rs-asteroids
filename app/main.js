import init, { Polygon } from './wasm/polygon.js';

const canvas = document.querySelector('#canvas')
const context = canvas.getContext('2d');

canvas.width = 400;
canvas.height = 400;
context.strokeStyle = '#666666';
context.lineWidth = 0.02;
context.lineCap = 'round';
context.lineJoin = 'round';

function drawPolygon(s, tx, ty, memory, polygon) {
    const length = polygon.length();
    const points = new Float64Array(memory.buffer, polygon.points(), length * 2);

    context.setTransform(s, 0, 0, s, tx, ty);
    context.beginPath();
    context.moveTo(points[0], points[1]);
    for (let i = 1; i < length; i += 1) {
        context.lineTo(points[i * 2], points[i * 2 + 1]);
    }
    context.closePath();
    context.stroke();
    context.setTransform(1, 0, 0, 1, 0, 0);
}

async function main() {
    const { memory } = await init();
    const hexagon = Polygon.ngon(6);

    drawPolygon(50, 50, 50, memory, hexagon);
    drawPolygon(30, 110, 110, memory, hexagon);
    drawPolygon(70, 200, 200, memory, hexagon);
    drawPolygon(30, 230, 230, memory, hexagon);
    drawPolygon(20, 350, 350, memory, hexagon);

    hexagon.free();
}

main();
