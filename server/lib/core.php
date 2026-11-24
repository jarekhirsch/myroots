<?php
/**
 * MyRoots Server: core (configuration, database, options, helpers).
 *
 * The same MyRoots app as on the Mac, Windows and in WordPress, here on its
 * own: PHP 8.0+ with SQLite, no other software. The browser runs the engine
 * (WebAssembly); the server stores the records, the photos and the published
 * family website, and checks who may do what.
 */

declare(strict_types=1);

const MR_VERSION = '3.1.0';
const MR_DB_VERSION = 1;

define('MR_ROOT', dirname(__DIR__));

// config.php (optional) may set MR_DATA: the data folder, best outside the public web folder.
if (is_file(MR_ROOT . '/config.php')) require MR_ROOT . '/config.php';
if (!defined('MR_DATA')) define('MR_DATA', MR_ROOT . '/data');

/* ---------------------------------------------------------------------- */
/*  Helpers                                                                 */
/* ---------------------------------------------------------------------- */

function mr_h(?string $s): string
{
    return htmlspecialchars((string) $s, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8');
}

function mr_random(int $len = 20): string
{
    $chars = 'abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789';
    $out = '';
    for ($i = 0; $i < $len; $i++) $out .= $chars[random_int(0, strlen($chars) - 1)];
    return $out;
}

/** Path of the installation on the server, always with a trailing slash ("/" or "/rodzina/"). */
function mr_base(): string
{
    static $b = null;
    if ($b === null) {
        $dir = str_replace('\\', '/', dirname($_SERVER['SCRIPT_NAME'] ?? '/'));
        $b = rtrim($dir, '/') . '/';
    }
    return $b;
}

/** Address inside the installation, e.g. mr_url('admin') → "/rodzina/admin". */
function mr_url(string $path = ''): string
{
    return mr_base() . ltrim($path, '/');
}

/** Full address with the host (e-mails). */
function mr_abs(string $path = ''): string
{
    $https = (!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off') || (($_SERVER['HTTP_X_FORWARDED_PROTO'] ?? '') === 'https');
    $host = preg_replace('/[^A-Za-z0-9.:\-\[\]]/', '', (string) ($_SERVER['HTTP_HOST'] ?? 'localhost'));
    return ($https ? 'https://' : 'http://') . $host . mr_url($path);
}

function mr_is_https(): bool
{
    return (!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off') || (($_SERVER['HTTP_X_FORWARDED_PROTO'] ?? '') === 'https');
}

/**
 * Ends the lock on the PHP session. PHP lets only one request of a person use
 * the session at a time, so without this the photos of a page would arrive
 * one after another instead of all at once. Call before sending a response;
 * nothing may be written to $_SESSION afterwards.
 */
function mr_release_session(): void
{
    if (session_status() === PHP_SESSION_ACTIVE) session_write_close();
}

function mr_json($data, int $status = 200): void
{
    mr_release_session();
    while (ob_get_level()) ob_end_clean();
    http_response_code($status);
    header('Content-Type: application/json; charset=UTF-8');
    header('Cache-Control: no-store');
    header('X-Content-Type-Options: nosniff');
    echo json_encode($data, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    exit;
}

function mr_error(string $code, string $message, int $status): void
{
    mr_json(['code' => $code, 'message' => $message, 'data' => ['status' => $status]], $status);
}

function mr_status(int $code): void
{
    http_response_code($code);
}

function mr_nocache(): void
{
    header('Cache-Control: no-cache, must-revalidate, max-age=0, no-store, private');
    header('Expires: Wed, 11 Jan 1984 05:00:00 GMT');
}

function mr_redirect(string $to): void
{
    header('Location: ' . $to, true, 302);
    exit;
}

function mr_body_json(): array
{
    $v = json_decode((string) file_get_contents('php://input'), true);
    return is_array($v) ? $v : [];
}

function mr_ip(): string
{
    return (string) ($_SERVER['REMOTE_ADDR'] ?? '');
}

/** "Kowalscy z Lublina" → "kowalscy-z-lublina" */
function mr_slugify(string $s): string
{
    $map = ['ą' => 'a', 'ć' => 'c', 'ę' => 'e', 'ł' => 'l', 'ń' => 'n', 'ó' => 'o', 'ś' => 's', 'ź' => 'z', 'ż' => 'z', 'ä' => 'a', 'ö' => 'o', 'ü' => 'u', 'ß' => 'ss', 'é' => 'e', 'è' => 'e', 'á' => 'a', 'à' => 'a', 'í' => 'i', 'ú' => 'u', 'ñ' => 'n', 'ç' => 'c', 'å' => 'a', 'æ' => 'ae', 'ø' => 'o', 'š' => 's', 'č' => 'c', 'ž' => 'z', 'ř' => 'r'];
    $s = mb_strtolower(trim($s), 'UTF-8');
    $s = strtr($s, $map);
    if (function_exists('iconv')) {
        $t = @iconv('UTF-8', 'ASCII//TRANSLIT//IGNORE', $s);
        if (is_string($t)) $s = $t;
    }
    $s = preg_replace('/[^a-z0-9]+/', '-', $s);
    return trim((string) $s, '-');
}

function mr_text(string $s, int $max = 200): string
{
    $s = trim(preg_replace('/[\x00-\x1F\x7F]+/u', ' ', $s) ?? '');
    return mb_substr(strip_tags($s), 0, $max, 'UTF-8');
}

/* ---------------------------------------------------------------------- */
/*  Database (SQLite)                                                       */
/* ---------------------------------------------------------------------- */

final class MR_Db
{
    private static ?PDO $pdo = null;

    public static function file(): string
    {
        return MR_DATA . '/myroots.sqlite';
    }

    public static function installed(): bool
    {
        return is_file(self::file()) && (int) MR_Opt::get('db_version', 0) > 0;
    }

    public static function pdo(): PDO
    {
        if (self::$pdo) return self::$pdo;
        if (!is_dir(MR_DATA)) @mkdir(MR_DATA, 0775, true);
        self::protect(MR_DATA);
        $pdo = new PDO('sqlite:' . self::file(), null, null, [
            PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
            PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
        ]);
        $pdo->exec('PRAGMA journal_mode = WAL');
        $pdo->exec('PRAGMA busy_timeout = 10000');
        $pdo->exec('PRAGMA foreign_keys = ON');
        $pdo->exec('CREATE TABLE IF NOT EXISTS options (k TEXT PRIMARY KEY, v TEXT NOT NULL)');
        self::$pdo = $pdo;
        return $pdo;
    }

    /** Closes a folder for direct access (Apache); other servers: see the nginx example. */
    public static function protect(string $dir): void
    {
        if (!is_dir($dir) || !is_writable($dir)) return;
        if (!is_file($dir . '/.htaccess')) @file_put_contents($dir . '/.htaccess', "Require all denied\nDeny from all\n");
        if (!is_file($dir . '/index.html')) @file_put_contents($dir . '/index.html', '');
    }

    public static function install(): void
    {
        $db = self::pdo();
        $db->exec('CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            login TEXT NOT NULL UNIQUE COLLATE NOCASE,
            name TEXT NOT NULL DEFAULT \'\',
            email TEXT NOT NULL DEFAULT \'\',
            pass TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT \'editor\',
            tree TEXT NOT NULL DEFAULT \'\',
            created INTEGER NOT NULL DEFAULT 0,
            last_login INTEGER NOT NULL DEFAULT 0
        )');
        $db->exec('CREATE TABLE IF NOT EXISTS attempts (ip TEXT NOT NULL, kind TEXT NOT NULL, t INTEGER NOT NULL)');
        $db->exec('CREATE INDEX IF NOT EXISTS attempts_ip ON attempts (ip, kind, t)');
        MR_Store::install_tree(MR_Trees::DEF);
        if (!MR_Opt::get('secret')) MR_Opt::set('secret', mr_random(20));
        MR_Opt::set('db_version', MR_DB_VERSION);
    }
}

/** Settings of the installation (one JSON value per key). */
final class MR_Opt
{
    private static array $cache = [];

    public static function get(string $k, $default = null)
    {
        if (array_key_exists($k, self::$cache)) return self::$cache[$k];
        $st = MR_Db::pdo()->prepare('SELECT v FROM options WHERE k = ?');
        $st->execute([$k]);
        $v = $st->fetchColumn();
        $out = $v === false ? $default : json_decode((string) $v, true);
        self::$cache[$k] = $out;
        return $out;
    }

    public static function set(string $k, $v): void
    {
        MR_Db::pdo()->prepare('INSERT INTO options (k, v) VALUES (?, ?) ON CONFLICT(k) DO UPDATE SET v = excluded.v')
            ->execute([$k, json_encode($v, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE)]);
        self::$cache[$k] = $v;
    }

    public static function delete(string $k): void
    {
        MR_Db::pdo()->prepare('DELETE FROM options WHERE k = ?')->execute([$k]);
        unset(self::$cache[$k]);
    }

    /** Short-lived values (publishing session, rate limits). */
    public static function get_temp(string $k)
    {
        $v = self::get('tmp_' . $k);
        if (!is_array($v) || ($v['until'] ?? 0) < time()) return null;
        return $v['v'];
    }

    public static function set_temp(string $k, $v, int $seconds): void
    {
        self::set('tmp_' . $k, ['v' => $v, 'until' => time() + $seconds]);
    }
}

/* ---------------------------------------------------------------------- */
/*  Language of the server pages (login, settings): Polish or English       */
/* ---------------------------------------------------------------------- */

function mr_lang(): string
{
    $l = (string) MR_Opt::get('lang', '');
    if ($l === '') {
        $al = strtolower((string) ($_SERVER['HTTP_ACCEPT_LANGUAGE'] ?? ''));
        $l = str_starts_with($al, 'pl') ? 'pl' : 'en';
    }
    return in_array($l, ['en', 'pl', 'de', 'fr', 'es', 'it', 'pt', 'nl', 'sv', 'fi', 'da', 'he'], true) ? $l : 'en';
}

function mr_t(string $en): string
{
    static $pl = null;
    if (mr_lang() !== 'pl') return $en;
    if ($pl === null) $pl = require __DIR__ . '/pl.php';
    return $pl[$en] ?? $en;
}
