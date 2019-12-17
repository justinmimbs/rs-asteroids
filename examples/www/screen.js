function screen(memory, screenCanvas) {
    const width = 1200;
    const height = 900;

    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    const screenContext = screenCanvas.getContext('2d');

    canvas.width = 3 * width;
    canvas.height = 3 * height;
    context.setTransform(1, 0, 0, 1, width, height);
    context.strokeStyle = '#666666';
    context.lineWidth = 1;
    context.lineCap = 'round';
    context.lineJoin = 'round';

    screenCanvas.style.background = '#eeeeee';
    screenCanvas.width = width;
    screenCanvas.height = height;

    function draw(list) {
        // draw to canvas
        const length = list.length();
        const paths = new Uint32Array(memory.buffer, list.paths(), length * 2);
        const alphas = new Float64Array(memory.buffer, list.alphas(), length);
        const ends = new Uint8Array(memory.buffer, list.ends(), length);
        const points = new Float64Array(memory.buffer, list.points(), list.points_length() * 2);

        context.clearRect(-width, -height, 3 * width, 3 * height);
        for (let i = 0; i < length; i += 1) {
            drawPoints(context, points, paths[i * 2], paths[i * 2 + 1], alphas[i], ends[i] === 1);
        }
        list.free();

        // copy to screenCanvas
        screenContext.clearRect(0, 0, width, height);
        for (let row in [ 0, 1, 2 ]) {
            for (let col in [ 0, 1, 2 ]) {
                screenContext.drawImage(canvas, col * width, row * height, width, height, 0, 0, width, height);
            }
        }
    }

    return {
        draw
    };
}

function drawPoints(context, points, index, length, alpha, isClosed) {
    context.beginPath();
    context.globalAlpha = alpha;
    context.moveTo(points[index * 2], points[index * 2 + 1]);
    for (let i = index + 1; i < index + length; i += 1) {
        context.lineTo(points[i * 2], points[i * 2 + 1]);
    }
    if (isClosed) {
        context.closePath();
    }
    context.stroke();
}

export { screen as default };
