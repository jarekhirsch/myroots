// MapLibre for Leaflet, loaded only when a map is shown. The worker script is
// bundled as text and started from a blob URL, so it works in the desktop app,
// on any hosting and in the family website opened from a disk.
import { setWorkerUrl } from 'maplibre-gl';
import worker from 'maplibre-gl/dist/maplibre-gl-worker.mjs?raw';
import 'maplibre-gl/dist/maplibre-gl.css';

setWorkerUrl(URL.createObjectURL(new Blob([worker], { type: 'text/javascript' })));

export { maplibreGL } from '@maplibre/maplibre-gl-leaflet';
