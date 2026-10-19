// Printing: the desktop app opens the system print dialog through Tauri
// (window.print() does nothing in the macOS web view).
import { backend } from './backend';

export function printPage() {
  if (backend.print) backend.print();
  else window.print();
}
