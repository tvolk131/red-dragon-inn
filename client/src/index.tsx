import * as React from 'react';
import {createRoot} from 'react-dom/client';
import {App} from './app';

const container = document.getElementById('app');
if (container) {
    createRoot(container).render(<App/>);
}
