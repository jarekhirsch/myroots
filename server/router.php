<?php
// For the PHP built-in server (testing on your own computer):
//   php -S localhost:8000 router.php
$p = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);
if ((preg_match('#^/(app|assets)/#', $p) || preg_match('#^/([a-z0-9][a-z0-9-]*/)?media/[0-9]+(_t)?\.[a-z0-9]+$#', $p)) && is_file(__DIR__ . $p)) {
    if (str_ends_with($p, '.wasm')) header('Content-Type: application/wasm');
    return false;
}
$_SERVER['SCRIPT_NAME'] = '/index.php';
require __DIR__ . '/index.php';
