<?php
/**
 * MyRoots Server: family trees and their data.
 *
 * Every tree has its own table of records (one JSON document per row, exactly
 * as the engine writes it), its own folder of photos (data/files-<tree>) and
 * of the published family website (data/site-<tree>), its own options and its
 * own address: the main tree at the main address, the others at /<name>/.
 */

declare(strict_types=1);

final class MR_Trees
{
    const DEF = 'default';
    /** Path names used by the server itself: never a tree address. */
    const RESERVED = ['admin', 'api', 'app', 'assets', 'data', 'login', 'logout', 'setup', 'embed', 'media', 'index', 'manifest', 'sw', 'editor', 'settings', 'index-php'];

    private static ?string $current = null;

    public static function all(): array
    {
        $t = MR_Opt::get('trees');
        if (!is_array($t) || !$t) $t = [self::DEF => ['name' => '', 'slug' => '', 'created' => 0]];
        return $t;
    }

    public static function exists(string $id): bool
    {
        return isset(self::all()[$id]);
    }

    /** The tree at the main address (the first one in the list). */
    public static function main(): string
    {
        $t = self::all();
        return isset($t[self::DEF]) ? self::DEF : (string) array_key_first($t);
    }

    public static function current(): string
    {
        if (self::$current === null || !self::exists(self::$current)) self::$current = self::main();
        return self::$current;
    }

    public static function set(string $id): bool
    {
        if (!self::exists($id)) return false;
        self::$current = $id;
        return true;
    }

    public static function suffix(?string $id = null): string
    {
        $id = $id ?? self::current();
        return $id === self::DEF ? '' : '_' . $id;
    }

    public static function get_opt(string $base, $default = null)
    {
        return MR_Opt::get($base . self::suffix(), $default);
    }

    public static function update_opt(string $base, $v): void
    {
        MR_Opt::set($base . self::suffix(), $v);
    }

    public static function name(?string $id = null): string
    {
        $id = $id ?? self::current();
        return (string) (self::all()[$id]['name'] ?? '');
    }

    /** Address segment of a tree; empty for the main tree. */
    public static function slug(?string $id = null): string
    {
        $id = $id ?? self::current();
        if ($id === self::main()) return '';
        $s = (string) (self::all()[$id]['slug'] ?? '');
        return $s !== '' ? $s : $id;
    }

    /** The tree for an address segment (also the old address of a tree that became the main one). */
    public static function by_slug(string $slug): ?string
    {
        if ($slug === '') return null;
        foreach (self::all() as $id => $t) {
            if ((string) $id !== self::main() && self::slug((string) $id) === $slug) return (string) $id;
        }
        foreach (self::all() as $id => $t) {
            if ((string) ($t['slug'] ?? '') === $slug) return (string) $id;
        }
        return null;
    }

    /** The tree last opened by the logged-in editor. */
    public static function for_user(): string
    {
        $id = MR_Auth::tree();
        return self::exists($id) ? $id : self::main();
    }

    private static function unique_slug(string $name, array $trees): string
    {
        $base = mr_slugify($name);
        if ($base === '' || in_array($base, self::RESERVED, true)) $base = 'tree-' . $base;
        $base = trim(substr($base, 0, 60), '-');
        $used = [];
        foreach ($trees as $t) $used[] = (string) ($t['slug'] ?? '');
        $s = $base;
        for ($n = 2; in_array($s, $used, true); $n++) $s = $base . '-' . $n;
        return $s;
    }

    public static function create(string $name): string
    {
        $trees = self::all();
        do {
            $id = 't' . base_convert((string) time(), 10, 36) . strtolower(mr_random(4));
            $id = preg_replace('/[^a-z0-9]/', '', $id);
        } while (isset($trees[$id]));
        $name = mr_text($name, 120);
        $trees[$id] = ['name' => $name, 'slug' => self::unique_slug($name, $trees), 'created' => time()];
        MR_Opt::set('trees', $trees);
        MR_Store::install_tree($id);
        return $id;
    }

    public static function rename(string $id, string $name): bool
    {
        $trees = self::all();
        if (!isset($trees[$id])) return false;
        $trees[$id]['name'] = mr_text($name, 120);
        MR_Opt::set('trees', $trees);
        return true;
    }

    public static function delete(string $id): bool
    {
        $trees = self::all();
        if (!isset($trees[$id]) || count($trees) < 2) return false;
        // public photos of its website (plain files next to index.php)
        $dir = MR_Site::mirror_dir($id);
        foreach ((array) glob($dir . '/*') as $f) if (is_string($f)) @unlink($f);
        @unlink($dir . '/.htaccess');
        @rmdir($dir);
        if (dirname($dir) !== MR_ROOT) @rmdir(dirname($dir));
        MR_Store::drop_tree($id);
        foreach (['gen', 'changed', 'site', 'auto_publish', 'inbox', 'weather_decided'] as $o) MR_Opt::delete($o . self::suffix($id));
        unset($trees[$id]);
        MR_Opt::set('trees', $trees);
        return true;
    }

    public static function export(): array
    {
        $out = [];
        $keep = self::current();
        foreach (self::all() as $id => $t) {
            $id = (string) $id;
            self::$current = $id;
            $out[] = [
                'id' => $id,
                'name' => (string) ($t['name'] ?? ''),
                'created' => (int) ($t['created'] ?? 0) * 1000,
                'slug' => self::slug($id),
                'url' => MR_Site::base(),
            ];
        }
        self::$current = $keep;
        return $out;
    }
}

final class MR_Store
{
    const KINDS = ['person', 'union', 'pet', 'source', 'media', 'place', 'revision', 'suggestion', 'settings', 'meta'];

    public static function table(?string $tree = null): string
    {
        return 'records' . MR_Trees::suffix($tree);
    }

    public static function install_tree(string $tree): void
    {
        $t = self::table($tree);
        MR_Db::pdo()->exec("CREATE TABLE IF NOT EXISTS \"$t\" (kind TEXT NOT NULL, rid INTEGER NOT NULL, data TEXT NOT NULL, updated INTEGER, PRIMARY KEY (kind, rid))");
        if (!MR_Opt::get('gen' . MR_Trees::suffix($tree))) MR_Opt::set('gen' . MR_Trees::suffix($tree), 1);
        self::dir('files', $tree);
        self::dir('site', $tree);
    }

    public static function drop_tree(string $tree): void
    {
        $t = self::table($tree);
        MR_Db::pdo()->exec("DROP TABLE IF EXISTS \"$t\"");
        foreach (['files', 'site'] as $what) self::rmdir(self::dir($what, $tree, false));
    }

    public static function rmdir(string $dir): void
    {
        if (!is_dir($dir)) return;
        $it = new RecursiveIteratorIterator(new RecursiveDirectoryIterator($dir, FilesystemIterator::SKIP_DOTS), RecursiveIteratorIterator::CHILD_FIRST);
        foreach ($it as $f) {
            if ($f->isDir()) @rmdir($f->getPathname());
            else @unlink($f->getPathname());
        }
        @rmdir($dir);
    }

    public static function dir(string $what, ?string $tree = null, bool $create = true): string
    {
        $tree = $tree ?? MR_Trees::current();
        $dir = MR_DATA . '/' . $what . '-' . $tree;
        if ($create && !is_dir($dir)) {
            @mkdir($dir, 0775, true);
            MR_Db::protect($dir);
        }
        return $dir;
    }

    /**
     * Sends all records of the open tree as a JSON array, row by row as they
     * are stored: nothing is decoded, so a tree of tens of thousands of people
     * needs only a few MB of PHP memory (hosting limits are often 128 MB).
     */
    public static function stream_all(): void
    {
        mr_release_session();
        $t = self::table();
        while (ob_get_level()) ob_end_clean();
        header('Content-Type: application/json; charset=UTF-8');
        header('Cache-Control: no-store');
        header('X-Content-Type-Options: nosniff');
        // JSON shrinks about ten times: compressed here when the server does not do it itself
        if (extension_loaded('zlib') && !ini_get('zlib.output_compression')) ob_start('ob_gzhandler');
        echo '[';
        $first = true;
        $check = function_exists('json_validate');
        foreach (MR_Db::pdo()->query("SELECT data FROM \"$t\"") as $row) {
            $d = trim((string) $row['data']);
            if ($d === '' || $d[0] !== '{' || ($check && !json_validate($d))) continue;
            echo $first ? '' : ',', $d;
            $first = false;
        }
        echo ']';
        exit;
    }

    /** All records of the open tree, decoded as objects (empty {} stay {}). */
    public static function all(): array
    {
        $t = self::table();
        $out = [];
        foreach (MR_Db::pdo()->query("SELECT data FROM \"$t\"") as $row) {
            $r = json_decode((string) $row['data'], false);
            if (is_object($r)) $out[] = $r;
        }
        return $out;
    }

    public static function get(string $kind, int $id): ?array
    {
        $t = self::table();
        $st = MR_Db::pdo()->prepare("SELECT data FROM \"$t\" WHERE kind = ? AND rid = ?");
        $st->execute([$kind, $id]);
        $json = $st->fetchColumn();
        if (!$json) return null;
        $r = json_decode((string) $json, true);
        return is_array($r) ? $r : null;
    }

    /** Applies a change set of the engine: [{kind, id, json}], json === null deletes. */
    public static function apply(array $records, array $deleted_media): int
    {
        $t = self::table();
        $db = MR_Db::pdo();
        $del = $db->prepare("DELETE FROM \"$t\" WHERE kind = ? AND rid = ?");
        $put = $db->prepare("INSERT OR REPLACE INTO \"$t\" (kind, rid, data, updated) VALUES (?, ?, ?, ?)");
        $n = 0;
        $db->beginTransaction();
        try {
            foreach ($records as $r) {
                if (!is_array($r)) continue;
                $kind = (string) ($r['kind'] ?? '');
                $id = isset($r['id']) ? (int) $r['id'] : -1;
                if (!in_array($kind, self::KINDS, true) || $id < 0) continue;
                $json = isset($r['json']) && is_string($r['json']) ? $r['json'] : null;
                if ($json === null) {
                    $del->execute([$kind, $id]);
                } else {
                    $check = json_decode($json, true);
                    if (!is_array($check) || ($check['kind'] ?? '') !== $kind || (int) ($check['id'] ?? -1) !== $id || !array_key_exists('data', $check)) continue;
                    $put->execute([$kind, $id, $json, time()]);
                }
                $n++;
            }
            $db->commit();
        } catch (Throwable $e) {
            $db->rollBack();
            throw $e;
        }
        foreach ($deleted_media as $mid) self::delete_media((int) $mid);
        if ($n) MR_Trees::update_opt('changed', time());
        return $n;
    }

    public static function reset(): int
    {
        $t = self::table();
        MR_Db::pdo()->exec("DELETE FROM \"$t\"");
        foreach ((array) glob(self::dir('files') . '/*.bin') as $f) if (is_string($f)) @unlink($f);
        $gen = (int) MR_Trees::get_opt('gen', 1) + 1;
        MR_Trees::update_opt('gen', $gen);
        MR_Trees::update_opt('changed', time());
        return $gen;
    }

    public static function media_path(int $id, bool $thumb): string
    {
        return self::dir('files') . '/' . $id . ($thumb ? '_t' : '') . '.bin';
    }

    public static function delete_media(int $id): void
    {
        foreach ([false, true] as $th) {
            $p = self::media_path($id, $th);
            if (is_file($p)) @unlink($p);
        }
    }

    /** Appends a piece of an upload; the file appears on the last piece. '' = ok, else error. */
    public static function write_chunk(string $final, string $bytes, int $offset, bool $last): string
    {
        $part = $final . '.part';
        if ($offset === 0) {
            if (is_file($part)) @unlink($part);
        } elseif (!is_file($part) || filesize($part) < $offset) {
            return 'upload pieces out of order';
        } elseif (filesize($part) > $offset) {
            // the same piece sent again after a lost answer: drop the first copy
            $h = fopen($part, 'r+');
            if (!$h || !ftruncate($h, $offset)) return 'cannot write to the data folder';
            fclose($h);
            clearstatcache(true, $part);
        }
        if (file_put_contents($part, $bytes, FILE_APPEND | LOCK_EX) === false) return 'cannot write to the data folder';
        clearstatcache(true, $part);
        if ($last && !rename($part, $final)) return 'cannot write to the data folder';
        return '';
    }

    /**
     * Sends a big text file (data.js of the family website, tens of MB for big
     * trees) compressed when the browser accepts it: a .gz copy is made once,
     * in pieces, next to the file and sent while the file does not change.
     */
    public static function send_text_gz(string $path): void
    {
        header('Vary: Accept-Encoding');
        $gzip = stripos((string) ($_SERVER['HTTP_ACCEPT_ENCODING'] ?? ''), 'gzip') !== false && function_exists('gzopen') && !ini_get('zlib.output_compression');
        if ($gzip) {
            $gz = $path . '.gz';
            if (!is_file($gz) || filemtime($gz) <= filemtime($path)) {
                $tmp = $gz . '.' . mr_random(6);
                $in = fopen($path, 'rb');
                $out = $in ? gzopen($tmp, 'wb6') : false;
                if ($in && $out) {
                    while (!feof($in)) gzwrite($out, (string) fread($in, 1 << 20));
                    gzclose($out);
                    fclose($in);
                    @rename($tmp, $gz);
                }
                if (is_file($tmp)) @unlink($tmp);
            }
            if (is_file($gz)) {
                header('Content-Encoding: gzip');
                header('Content-Length: ' . filesize($gz));
                readfile($gz);
                return;
            }
        }
        header('Content-Length: ' . filesize($path));
        readfile($path);
    }

    public static function send_file(string $path, string $mime, bool $private): void
    {
        if (!is_file($path)) {
            // not kept by browsers and Cloudflare: the photo may come with the next publication
            header('Cache-Control: no-store');
            mr_status(404);
            exit;
        }
        mr_release_session();
        $size = filesize($path);
        $etag = '"' . md5($path . filemtime($path) . $size) . '"';
        while (ob_get_level()) ob_end_clean();
        header_remove('Expires');
        header_remove('Pragma');
        header('Content-Type: ' . $mime);
        header('X-Content-Type-Options: nosniff');
        header('Cache-Control: ' . ($private ? 'private' : 'public') . ', max-age=31536000, immutable');
        header('ETag: ' . $etag);
        if (trim((string) ($_SERVER['HTTP_IF_NONE_MATCH'] ?? '')) === $etag) {
            mr_status(304);
            exit;
        }
        header('Content-Length: ' . $size);
        readfile($path);
        exit;
    }
}
