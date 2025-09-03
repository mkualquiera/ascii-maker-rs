// worker.js
import init, { process_image } from './out/ascii_worker.js';

let initialized = false;

self.onmessage = async function (e) {
    try {
        if (!initialized) {
            await init();
            initialized = true;
            console.log('WASM initialized');
        }

        const callback = (data) => {
            self.postMessage({ type: 'stream', data });
        };

        console.log('Processing image, data length:', e.data.length);
        const result = process_image(e.data.data, e.data.cols, e.data.invert, callback);
        self.postMessage({ type: 'done', result });

    } catch (error) {
        console.error('Worker error:', error);
        self.postMessage({ type: 'error', error: error.message });
    }
};

self.onerror = (error) => {
    console.error('Worker script error:', error);
};