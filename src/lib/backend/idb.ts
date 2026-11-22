// Minimal promise wrapper around IndexedDB.
const VERSION = 1;
let DB_NAME = 'myroots';

let dbp: Promise<IDBDatabase> | null = null;

/** The first tree keeps the original database name, the others get their own. */
export function dbNameFor(tree: string): string {
  return tree === 'default' ? 'myroots' : `myroots-${tree}`;
}

export function useTree(tree: string) {
  const name = dbNameFor(tree);
  if (name === DB_NAME) return;
  if (dbp) dbp.then((db) => db.close()).catch(() => {});
  dbp = null;
  DB_NAME = name;
}

export function deleteDb(tree: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.deleteDatabase(dbNameFor(tree));
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
    req.onblocked = () => resolve();
  });
}

export function openDb(): Promise<IDBDatabase> {
  if (dbp) return dbp;
  dbp = new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains('records')) db.createObjectStore('records');
      if (!db.objectStoreNames.contains('blobs')) db.createObjectStore('blobs');
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
  return dbp;
}

function done(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
    tx.onabort = () => reject(tx.error);
  });
}

export async function getAll(store: string): Promise<any[]> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const req = db.transaction(store).objectStore(store).getAll();
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export async function get(store: string, key: string): Promise<any> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const req = db.transaction(store).objectStore(store).get(key);
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export async function keys(store: string): Promise<string[]> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const req = db.transaction(store).objectStore(store).getAllKeys();
    req.onsuccess = () => resolve(req.result as string[]);
    req.onerror = () => reject(req.error);
  });
}

/** Applies puts (value !== undefined) and deletes (value === undefined) in one transaction. */
export async function write(store: string, ops: [string, any][]): Promise<void> {
  if (!ops.length) return;
  const db = await openDb();
  const tx = db.transaction(store, 'readwrite');
  const os = tx.objectStore(store);
  for (const [k, v] of ops) {
    if (v === undefined) os.delete(k);
    else os.put(v, k);
  }
  return done(tx);
}

export async function clear(store: string): Promise<void> {
  const db = await openDb();
  const tx = db.transaction(store, 'readwrite');
  tx.objectStore(store).clear();
  return done(tx);
}
