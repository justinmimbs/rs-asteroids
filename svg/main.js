import init, { App } from '../wasm/app.js';

const width = 1200;
const height = 900;

const screen = node('svg',
    {
        width,
        height,
        viewBox: `0 0 ${width} ${height}`,
        style: 'background: radial-gradient(farthest-corner circle at center, #21272D, #17191B);',
        fill: 'none',
        stroke: '#EAF9FF',
        'stroke-linecap': 'round',
        'stroke-linejoin': 'round',
    },
    []
);

let app;
let memory;
let time;
let looping = 0;

const frames = [];
const slider = document.querySelector('input');

main();

async function main() {
    const container = document.querySelector('main');
    container.insertBefore(screen, container.firstChild);

    window.addEventListener('keydown', handleKey(true));
    window.addEventListener('keyup', handleKey(false));
    window.addEventListener('keydown', function(event) {
        if (event.key === ' ') {
            event.preventDefault();
            playPause();
        }
    });
    slider.addEventListener('input', (_) => scrub(slider.value));

    const wasm = await init();
    memory = wasm.memory;
    app = App.new();

    playPause();
}

function playPause() {
    if (!looping) {
        time = performance.now();
        loop(time);
        slider.style.visibility = 'hidden';
    } else {
        cancelAnimationFrame(looping);
        looping = 0;
        slider.style.visibility = 'visible';
        slider.max = frames.length - 1;
        slider.value = slider.max;
        scrub(slider.value);
    }
}

function scrub(i) {
    if (!looping && frames[i]) {
        draw(frames[i]);
    }
}

function loop(now) {
    app.step((now - time) / 1000, bitpackControls());
    time = now;
    looping = requestAnimationFrame(loop);

    if (600 <= frames.length) {
        frames.shift();
    }
    frames.push(screenData());
    draw(frames[frames.length - 1]);
}

// controls

const controls = {
    left: false,
    right: false,
    thrust: false,
    fire: false,
    shield: false,
    start: false,
};

function bitpackControls() {
    return 0
        + (controls.left ? 1 : 0)
        + (controls.right ? 2 : 0)
        + (controls.thrust ? 4 : 0)
        + (controls.fire ? 8 : 0)
        + (controls.shield ? 16 : 0)
        + (controls.start ? 32 : 0);
}

function handleKey(down) {
    return function (event) {
        let control = keyToControl(event.key);
        if (control !== null) {
            controls[control] = down;
        }
    }
}

function keyToControl(key) {
    switch (event.key) {
        case 'ArrowLeft':
            return 'left';

        case 'ArrowRight':
            return 'right';

        case 'ArrowUp':
            return 'thrust';

        case 's':
        case 'S':
            return 'shield';

        case 'f':
        case 'F':
            return 'fire';

        case 'Enter':
            return 'start';

        default:
            return null;
    }
}

// drawing

function draw(data) {
    // clear
    while (screen.firstChild) {
        screen.removeChild(screen.firstChild);
    }

    // drawing
    const id = 'asteroids-drawing';
    const g = node('g', { id }, []);
    for (let d of data) {
        g.appendChild(
            node(d.isClosed ? 'polygon' : 'polyline',
                {
                    points: d.points.join(' '),
                    opacity: d.alpha,
                    'stroke-width': 0.6 + d.alpha,
                },
                []
            )
        );
    }
    screen.appendChild(node('defs', {}, [ g ]));
    screen.appendChild(node('use', { href: '#' + id }, []));

    // effects
    if (!looping) {
        const blur1 = 'asteroids-blur1';
        const blur2 = 'asteroids-blur2';
        screen.appendChild(
            node('defs',
                {},
                [
                    node('filter',
                        { id: blur1 },
                        [ node('feGaussianBlur', { stdDeviation: '3' }, []) ]
                    ),
                    node('filter',
                        { id: blur2 },
                        [ node('feGaussianBlur', { stdDeviation: '20' }, []) ]
                    ),
                ]
            )
        );
        screen.appendChild(node('use', { href: '#' + id, filter: 'url(#' + blur2 + ')', opacity: '0.4' }, []));
        screen.appendChild(node('use', { href: '#' + id, filter: 'url(#' + blur1 + ')', opacity: '0.4' }, []));
    }
}

function node(name, attributes, children) {
    const n = document.createElementNS('http://www.w3.org/2000/svg', name);
    for (let key of Object.keys(attributes)) {
        n.setAttribute(key, attributes[key]);
    }
    for (let child of children) {
        n.appendChild(child);
    }
    return n;
}

function screenData() {
    // render
    const list = app.render();
    const length = list.length();
    const paths = new Uint32Array(memory.buffer, list.paths(), length * 2);
    const alphas = new Float64Array(memory.buffer, list.alphas(), length);
    const ends = new Uint8Array(memory.buffer, list.ends(), length);
    const points = new Float64Array(memory.buffer, list.points(), list.points_length() * 2);

    // data
    const data = [];
    for (let i = 0; i < length; i += 1) {
        data.push(pointsData(points, paths[i * 2], paths[i * 2 + 1], alphas[i], ends[i] === 1));
    }
    list.free();

    return data;
}

function pointsData(array, index, length, alpha, isClosed) {
    const points = [];
    for (let i = index; i < index + length; i += 1) {
        points.push(array[i * 2]);
        points.push(array[i * 2 + 1]);
    }
    return { points, alpha, isClosed };
}
