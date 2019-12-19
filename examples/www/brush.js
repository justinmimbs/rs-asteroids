const brushing = new Map(); // element -> initial, forall elements being brushed
const listeners = new WeakMap(); // element -> Set(listener)

function handleDown(event) {
    if (brushing.size === 0) {
        window.addEventListener('pointermove', handleMove);
        window.addEventListener('pointerup', handleUp);
    }
    let element = event.currentTarget;
    let initial = {
        offsetX: event.offsetX,
        offsetY: event.offsetY,
        pageX: event.pageX,
        pageY: event.pageY,
    };
    brushing.set(element, initial);
    notifyListeners(element, initial, event, 'start');
}

function handleMove(event) {
    brushing.forEach(function (initial, element) {
        notifyListeners(element, initial, event, 'move');
    });
}

function handleUp(event) {
    brushing.forEach(function (initial, element) {
        notifyListeners(element, initial, event, 'end');
    });
    brushing.clear();
    window.removeEventListener('pointermove', handleMove);
    window.removeEventListener('pointerup', handleUp);
}

function notifyListeners(element, initial, event, phase) {
    if (listeners.has(element)) {
        let data = {
            phase: phase,
            initialX: initial.offsetX,
            initialY: initial.offsetY,
            extentX: event.pageX - initial.pageX,
            extentY: event.pageY - initial.pageY,
        };
        listeners.get(element).forEach(function (listener) {
            listener(data);
        });
    }
}

function addBrushListener(element, listener) {
    if (typeof listener !== 'function') {
        return;
    }
    if (listeners.has(element) && !listeners.get(element).has(listener)) {
        listeners.get(element).add(listener);
    } else {
        listeners.set(element, new Set([ listener ]));
        element.addEventListener('pointerdown', handleDown);
    }
}

function removeBrushListener(element, listener) {
    if (typeof listener !== 'function') {
        return;
    }
    if (listeners.has(element) && listeners.get(element).has(listener)) {
        listeners.get(element).delete(listener);
    }
}

export { addBrushListener, removeBrushListener }
