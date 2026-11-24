<?php
/**
 * MyRoots Server: every address goes through this file (see .htaccess).
 *
 *   /               family website of the main tree      /<name>/   other trees
 *   /admin          the app (editors)                    /admin/settings  server settings, accounts
 *   /login /logout  logging in                           /setup     first start
 *   /api/...        the API of the app
 */

declare(strict_types=1);

require __DIR__ . '/lib/core.php';
require __DIR__ . '/lib/auth.php';
require __DIR__ . '/lib/store.php';
require __DIR__ . '/lib/api.php';
require __DIR__ . '/lib/site.php';
require __DIR__ . '/lib/pages.php';

// the path inside the installation
$uri = (string) parse_url((string) ($_SERVER['REQUEST_URI'] ?? '/'), PHP_URL_PATH);
$uri = rawurldecode($uri);
$base = mr_base();
$path = str_starts_with($uri, $base) ? substr($uri, strlen($base)) : ltrim($uri, '/');
if (str_starts_with($path, 'index.php')) $path = ltrim(substr($path, 9), '/');
if (str_contains($path, '..') || str_contains($path, "\0")) {
    http_response_code(400);
    exit;
}

header('X-Content-Type-Options: nosniff');

// first start
if ($path === 'setup' || !is_file(MR_Db::file()) || !MR_Db::installed() || MR_Auth::count() === 0) {
    if ($path !== 'setup' && str_starts_with($path, 'api/')) mr_error('not_installed', 'MyRoots is not installed yet', 503);
    if ($path !== 'setup') mr_redirect(mr_url('setup'));
    MR_Pages::setup();
}

try {
    switch (true) {
        case $path === 'login':
            MR_Pages::login();
        case $path === 'logout':
            MR_Pages::logout();
        case $path === 'admin' || $path === 'admin/':
            MR_Site::editor();
        case $path === 'admin/settings':
            MR_Pages::settings();
        case $path === 'editor.webmanifest':
            MR_Pwa::editor_manifest();
        case str_starts_with($path, 'api/'):
            MR_Api::dispatch(substr($path, 4));
        case str_starts_with($path, 'data/') || str_starts_with($path, 'lib/'):
            http_response_code(404);
            exit;
        default:
            MR_Site::serve($path);
    }
} catch (Throwable $e) {
    error_log('MyRoots: ' . $e->getMessage() . ' in ' . $e->getFile() . ':' . $e->getLine());
    if (str_starts_with($path, 'api/')) mr_error('server_error', 'server error: ' . $e->getMessage(), 500);
    http_response_code(500);
    echo 'MyRoots: server error. See the PHP error log.';
}
