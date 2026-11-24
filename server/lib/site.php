<?php
/**
 * MyRoots Server: the family website (read-only), the editor page and the
 * installable app (PWA).
 *
 * The website is the MyRoots app itself in read-only mode, with the data the
 * editor published (the engine applies the privacy settings and the password
 * protection before anything leaves the browser).
 */

declare(strict_types=1);

final class MR_Site
{
    /** Address of the website of the current tree: "/" or "/kowalscy/" (inside the installation). */
    public static function base(): string
    {
        $s = MR_Trees::slug();
        return mr_url($s === '' ? '' : $s . '/');
    }

    public static function state(): array
    {
        return (array) MR_Trees::get_opt('site', []);
    }

    public static function visibility(): string
    {
        $v = (string) MR_Opt::get('visibility', 'public');
        return in_array($v, ['public', 'users', 'off'], true) ? $v : 'public';
    }

    /** Who may see the website: everybody, people with an account, or only the editors. */
    public static function visible(): bool
    {
        $v = self::visibility();
        if ($v === 'public') return true; // no session: photos of a public site are served without it
        if (MR_Auth::can_edit()) return true;
        if ($v === 'users') return MR_Auth::user() !== null;
        return false;
    }

    /** Files of the built app (entry module and stylesheets) from app/index.html. */
    public static function assets(): array
    {
        $html = (string) @file_get_contents(MR_ROOT . '/app/index.html');
        $out = ['js' => [], 'css' => []];
        if (preg_match_all('#<script[^>]+src="\./(assets/[A-Za-z0-9._-]+\.js)"#', $html, $m)) $out['js'] = $m[1];
        if (preg_match_all('#<link[^>]+rel="stylesheet"[^>]+href="\./(assets/[A-Za-z0-9._-]+\.css)"#', $html, $m)) $out['css'] = $m[1];
        return $out;
    }

    /** Switches to the uploaded version and removes photos that are no longer public. */
    public static function finish(array $sigs): string
    {
        $dir = MR_Store::dir('site');
        $next = $dir . '/data.next.js';
        if (!is_file($next)) return 'data missing';
        $js = (string) file_get_contents($next);
        $persons = substr_count($js, '{"kind":"person"');
        if (preg_match('#"lock":\{"v":1,"mode":"all".*?"people":(\d+)#s', $js, $lm)) $persons = (int) $lm[1];
        if (!rename($next, $dir . '/data.js')) return 'cannot write';
        foreach ((array) glob($dir . '/media/*') as $f) {
            if (!is_string($f)) continue;
            $id = (int) basename($f);
            if (!isset($sigs[$id]) || substr($f, -5) === '.part') @unlink($f);
        }
        MR_Trees::update_opt('site', ['published' => time(), 'sigs' => $sigs, 'persons' => $persons]);
        self::mirror_all();
        return '';
    }

    /**
     * Photos of a public website also as plain files next to index.php
     * (media/ for the main tree, <address>/media/ for the others). The web
     * server then sends them itself, which is many times faster than a PHP
     * request per photo on shared hosting. A website for logged-in people only
     * keeps them in data/ and sends them through PHP, after the login check.
     */
    public static function mirror_dir(string $tree): string
    {
        $s = MR_Trees::slug($tree);
        return MR_ROOT . '/' . ($s === '' ? '' : $s . '/') . 'media';
    }

    public static function mirror(string $tree): void
    {
        $dst = self::mirror_dir($tree);
        $want = [];
        if (self::visibility() === 'public') {
            $src = MR_Store::dir('site', $tree, false) . '/media';
            foreach ((array) glob($src . '/*') as $f) {
                if (is_string($f) && is_file($f) && substr($f, -5) !== '.part') $want[basename($f)] = $f;
            }
        }
        if (!is_dir($dst)) {
            if (!$want || !@mkdir($dst, 0755, true)) return;
        }
        foreach ((array) glob($dst . '/*') as $f) {
            if (is_string($f) && !isset($want[basename($f)])) @unlink($f);
        }
        if (!$want) {
            @unlink($dst . '/.htaccess');
            @rmdir($dst);
            $up = dirname($dst);
            if ($up !== MR_ROOT) @rmdir($up); // the folder of a tree address, when empty
            return;
        }
        $ht = $dst . '/.htaccess';
        if (!is_file($ht)) {
            @file_put_contents($ht, "# Written by MyRoots: public photos of the family website.\nOptions -Indexes\n<IfModule mod_headers.c>\n  Header set Cache-Control \"public, max-age=31536000, immutable\"\n  Header set X-Content-Type-Options \"nosniff\"\n  Header set Content-Security-Policy \"default-src 'none'; sandbox\"\n</IfModule>\n");
        }
        foreach ($want as $name => $f) {
            $t = $dst . '/' . $name;
            if (is_file($t) && filesize($t) === filesize($f) && filemtime($t) >= filemtime($f)) continue;
            @unlink($t);
            if (!@link($f, $t) && !@copy($f, $t)) continue;
            @chmod($t, 0644);
        }
    }

    /** All trees (after publishing, a change of who can see the website, a deleted tree). */
    public static function mirror_all(): void
    {
        foreach (array_keys(MR_Trees::all()) as $id) self::mirror((string) $id);
    }

    /** Requests for the family website: page, data, photos, app files of the PWA. */
    public static function serve(string $path): void
    {
        // the tree from the first part of the address
        $parts = explode('/', $path, 2);
        $id = MR_Trees::by_slug($parts[0]);
        if ($id !== null) {
            MR_Trees::set($id);
            $path = $parts[1] ?? '';
            if (!isset($parts[1])) mr_redirect(self::base());
        } else {
            MR_Trees::set(MR_Trees::main());
        }
        if ($path === 'index' || $path === 'index.html') $path = '';
        if ($path === 'manifest.webmanifest') MR_Pwa::site_manifest();
        if ($path === 'sw.js') MR_Pwa::service_worker();

        if (!self::visible()) {
            if ($path === '' || $path === 'embed') {
                if (self::visibility() === 'users' || MR_Auth::user() === null) mr_redirect(mr_url('login') . '?next=' . rawurlencode(self::base()));
                self::message(mr_t('The family website is not available.'), '');
            }
            mr_status(403);
            exit;
        }
        $dir = MR_Store::dir('site');
        $private = self::visibility() !== 'public';
        if ($path === 'data.js') {
            mr_release_session();
            while (ob_get_level()) ob_end_clean();
            header('Content-Type: application/javascript; charset=UTF-8');
            header('Cache-Control: ' . ($private ? 'private' : 'public') . ', max-age=60');
            header('X-Content-Type-Options: nosniff');
            $f = $dir . '/data.js';
            if (is_file($f)) MR_Store::send_text_gz($f);
            else echo 'window.MYROOTS_SITE=null;';
            exit;
        }
        if (preg_match('#^media/(\d+(?:_t)?)\.([a-z0-9]{2,5})$#', $path, $m)) {
            $types = ['jpg' => 'image/jpeg', 'jpeg' => 'image/jpeg', 'png' => 'image/png', 'webp' => 'image/webp', 'gif' => 'image/gif', 'avif' => 'image/avif', 'heic' => 'image/heic', 'pdf' => 'application/pdf', 'mp3' => 'audio/mpeg', 'm4a' => 'audio/mp4', 'ogg' => 'audio/ogg', 'oga' => 'audio/ogg', 'opus' => 'audio/ogg', 'wav' => 'audio/wav', 'webm' => 'audio/webm', 'mp4' => 'video/mp4', 'aac' => 'audio/aac', 'flac' => 'audio/flac'];
            header("Content-Security-Policy: default-src 'none'; sandbox");
            // a public website published before the photos became plain files: make them now
            if (!$private && !is_dir(self::mirror_dir(MR_Trees::current()))) self::mirror(MR_Trees::current());
            MR_Store::send_file($dir . '/media/' . $m[1] . '.' . $m[2], $types[$m[2]] ?? 'application/octet-stream', $private);
        }
        if ($path === '' || $path === 'embed') self::page($path === 'embed');
        mr_status(404);
        self::message(mr_t('This page does not exist.'), '');
    }

    /** A short message page in the style of the app. */
    public static function message(string $title, string $html): void
    {
        mr_nocache();
        header('Content-Type: text/html; charset=UTF-8');
        MR_Pages::shell($title, '<div class="card center"><img src="' . mr_h(mr_url('app/icon.svg')) . '" alt="" width="56" height="56"><h1>' . mr_h($title) . '</h1>' . $html . '</div>');
        exit;
    }

    private static function page(bool $embed): void
    {
        $st = self::state();
        $view = mr_url('app/');
        $files = self::assets();
        $title = MR_Trees::name() !== '' ? MR_Trees::name() : (string) MR_Opt::get('title', 'MyRoots');
        if (empty($st['published'])) {
            $link = MR_Auth::can_edit() ? '<p><a class="btn" href="' . mr_h(mr_url('admin?tree=' . MR_Trees::current() . '#/website')) . '">' . mr_h(mr_t('Open MyRoots and publish it')) . '</a></p>' : '';
            self::message(mr_t('The family tree is not published yet.'), $link);
        }
        $cfg = [
            'media' => self::base() . 'media/',
            'home' => '',
            'site_name' => $title,
            'suggest' => mr_url('api/suggest') . '?tree=' . rawurlencode(MR_Trees::current()),
        ];
        if (MR_Auth::can_edit() && !$embed) {
            $cfg['home'] = mr_url('admin?tree=' . MR_Trees::current());
            $cfg['site_name'] = mr_t('Edit the tree');
        }
        $ver = MR_VERSION . '.' . (int) ($st['published'] ?? 0);
        $flags = JSON_HEX_TAG | JSON_HEX_AMP | JSON_HEX_APOS | JSON_HEX_QUOT | JSON_UNESCAPED_SLASHES;
        mr_nocache();
        header('Content-Type: text/html; charset=UTF-8');
        header('X-Content-Type-Options: nosniff');
        header('Referrer-Policy: same-origin');
        if (!$embed) header('X-Frame-Options: SAMEORIGIN');
        $index = (bool) MR_Opt::get('index', false) && self::visibility() === 'public';
        echo "<!doctype html>\n<html lang=\"" . mr_h(mr_lang()) . "\">\n<head>\n<meta charset=\"UTF-8\" />\n";
        echo "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\" />\n";
        if (!$index || $embed) echo "<meta name=\"robots\" content=\"noindex\" />\n";
        echo '<title>' . mr_h($title) . "</title>\n";
        echo '<link rel="icon" type="image/svg+xml" href="' . mr_h($view . 'icon.svg') . "\" />\n";
        if (!$embed) MR_Pwa::head_tags($title);
        foreach ($files['css'] as $css) echo '<link rel="stylesheet" href="' . mr_h($view . $css) . "\" />\n";
        if ($embed) echo "<style>html,body{background:transparent}</style>\n";
        echo '<script>window.MYROOTS_ASSETS=' . json_encode($view, $flags) . ';window.MYROOTS_WP_VIEW=' . json_encode($cfg, $flags) . ";</script>\n";
        echo '<script src="' . mr_h(self::base() . 'data.js?v=' . $ver) . "\"></script>\n";
        if (!$embed) MR_Pwa::register_script();
        foreach ($files['js'] as $js) echo '<script type="module" src="' . mr_h($view . $js) . "\"></script>\n";
        echo "</head>\n<body>\n<div id=\"app\"></div>\n</body>\n</html>\n";
        exit;
    }

    /** The editor: the full app, working on the server through the API. */
    public static function editor(): void
    {
        if (!MR_Auth::can_edit()) {
            if (MR_Auth::user()) self::message(mr_t('Your account can only view the family website.'), '<p><a class="btn" href="' . mr_h(mr_url('')) . '">' . mr_h(mr_t('Go to the family website')) . '</a> <a class="btn ghost" href="' . mr_h(mr_url('logout')) . '">' . mr_h(mr_t('Log out')) . '</a></p>');
            mr_redirect(mr_url('login') . '?next=' . rawurlencode(mr_url('admin')));
        }
        $want = preg_replace('/[^a-z0-9]/', '', (string) ($_GET['tree'] ?? ''));
        if ($want !== '' && MR_Trees::exists($want)) MR_Auth::remember_tree($want);
        MR_Trees::set(MR_Trees::for_user());
        $st = self::state();
        $app = mr_url('app/');
        $u = MR_Auth::user();
        $cfg = [
            'host' => 'server',
            'rest' => mr_url('api/'),
            'nonce' => MR_Auth::csrf(),
            'ajax' => mr_url('api/nonce'),
            'assets' => $app,
            'home' => mr_url(''),
            'admin' => MR_Auth::is_admin() ? mr_url('admin/settings') : self::base(),
            'back_label' => MR_Auth::is_admin() ? mr_t('Server settings') : mr_t('Family website'),
            'logout' => mr_url('logout'),
            'site_url' => self::base(),
            'site_name' => (string) MR_Opt::get('title', 'MyRoots'),
            'gen' => (int) MR_Trees::get_opt('gen', 1),
            'tree' => MR_Trees::current(),
            'tree_slug' => MR_Trees::slug(),
            'legacy' => 0,
            'can_publish' => true,
            'auto_publish' => (bool) MR_Trees::get_opt('auto_publish', true),
            'published' => (int) ($st['published'] ?? 0) * 1000,
            'user' => (string) ($u['name'] ?: $u['login']),
            'lang' => mr_lang(),
            'weather_decided' => (bool) MR_Trees::get_opt('weather_decided', false),
        ];
        $files = self::assets();
        $flags = JSON_HEX_TAG | JSON_HEX_AMP | JSON_HEX_APOS | JSON_HEX_QUOT | JSON_UNESCAPED_SLASHES;
        mr_nocache();
        header('Content-Type: text/html; charset=UTF-8');
        header('X-Frame-Options: SAMEORIGIN');
        header('X-Content-Type-Options: nosniff');
        header('Referrer-Policy: same-origin');
        echo "<!doctype html>\n<html lang=\"" . mr_h(mr_lang()) . "\">\n<head>\n<meta charset=\"UTF-8\" />\n";
        echo "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\" />\n";
        echo "<meta name=\"robots\" content=\"noindex\" />\n<meta name=\"theme-color\" content=\"#1d5a44\" />\n";
        echo '<title>' . mr_h('MyRoots · ' . $cfg['site_name']) . "</title>\n";
        echo '<link rel="icon" type="image/svg+xml" href="' . mr_h($app . 'icon.svg') . "\" />\n";
        echo '<link rel="apple-touch-icon" href="' . mr_h($app . 'apple-touch-icon.png') . "\" />\n";
        echo '<link rel="manifest" href="' . mr_h(mr_url('editor.webmanifest')) . "\" />\n";
        echo "<meta name=\"apple-mobile-web-app-capable\" content=\"yes\" />\n<meta name=\"mobile-web-app-capable\" content=\"yes\" />\n<meta name=\"apple-mobile-web-app-title\" content=\"MyRoots\" />\n";
        foreach ($files['css'] as $css) echo '<link rel="stylesheet" href="' . mr_h($app . $css) . "\" />\n";
        echo '<script>window.MYROOTS_WP=' . json_encode($cfg, $flags) . ";</script>\n";
        foreach ($files['js'] as $js) echo '<script type="module" src="' . mr_h($app . $js) . "\"></script>\n";
        echo "</head>\n<body>\n<div id=\"app\"></div>\n</body>\n</html>\n";
        exit;
    }
}

final class MR_Pwa
{
    const THEME = '#1d5a44';
    const BG = '#f6f8f6';

    private static function icons(): array
    {
        $app = mr_url('app/');
        return [
            ['src' => $app . 'icon-192.png', 'sizes' => '192x192', 'type' => 'image/png'],
            ['src' => $app . 'icon-512.png', 'sizes' => '512x512', 'type' => 'image/png'],
            ['src' => $app . 'icon-512.png', 'sizes' => '512x512', 'type' => 'image/png', 'purpose' => 'maskable'],
            ['src' => $app . 'icon.svg', 'sizes' => 'any', 'type' => 'image/svg+xml'],
        ];
    }

    private static function send(array $m): void
    {
        while (ob_get_level()) ob_end_clean();
        header('Content-Type: application/manifest+json; charset=UTF-8');
        header('Cache-Control: public, max-age=3600');
        echo json_encode($m, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
        exit;
    }

    public static function site_manifest(): void
    {
        $name = MR_Trees::name() !== '' ? MR_Trees::name() : (string) MR_Opt::get('title', 'MyRoots');
        $start = MR_Site::base();
        self::send([
            'id' => $start, 'name' => $name, 'short_name' => mb_substr($name, 0, 24), 'description' => mr_t('Family tree'),
            'lang' => mr_lang(), 'start_url' => $start, 'scope' => $start, 'display' => 'standalone',
            'background_color' => self::BG, 'theme_color' => self::THEME, 'icons' => self::icons(),
        ]);
    }

    public static function editor_manifest(): void
    {
        $start = mr_url('admin');
        self::send([
            'id' => $start, 'name' => 'MyRoots · ' . (string) MR_Opt::get('title', ''), 'short_name' => 'MyRoots', 'description' => mr_t('Edit the tree'),
            'lang' => mr_lang(), 'start_url' => $start, 'scope' => mr_url(''), 'display' => 'standalone',
            'background_color' => self::BG, 'theme_color' => self::THEME, 'icons' => self::icons(),
        ]);
    }

    public static function service_worker(): void
    {
        $st = MR_Site::state();
        $cfg = [
            'cache' => 'myroots-' . MR_Trees::current() . '-' . MR_VERSION . '-' . (int) ($st['published'] ?? 0),
            'prefix' => 'myroots-' . MR_Trees::current() . '-',
            'start' => MR_Site::base(),
            'app' => mr_url('app/'),
            'media' => MR_Site::base() . 'media/',
            'skip' => [mr_url('admin'), mr_url('api/'), mr_url('login'), mr_url('logout'), mr_url('setup')],
        ];
        while (ob_get_level()) ob_end_clean();
        header('Content-Type: application/javascript; charset=UTF-8');
        header('Cache-Control: no-cache');
        header('Service-Worker-Allowed: ' . MR_Site::base());
        echo 'const C=' . json_encode($cfg, JSON_UNESCAPED_SLASHES) . ";\n";
        readfile(MR_ROOT . '/assets/sw.js');
        exit;
    }

    public static function head_tags(string $title): void
    {
        echo '<link rel="manifest" href="' . mr_h(MR_Site::base() . 'manifest.webmanifest') . "\" />\n";
        echo '<meta name="theme-color" content="' . self::THEME . "\" />\n";
        echo "<meta name=\"apple-mobile-web-app-capable\" content=\"yes\" />\n<meta name=\"mobile-web-app-capable\" content=\"yes\" />\n";
        echo '<meta name="apple-mobile-web-app-title" content="' . mr_h($title) . "\" />\n";
        echo '<link rel="apple-touch-icon" href="' . mr_h(mr_url('app/apple-touch-icon.png')) . "\" />\n";
    }

    public static function register_script(): void
    {
        $sw = json_encode(MR_Site::base() . 'sw.js', JSON_UNESCAPED_SLASHES);
        $scope = json_encode(MR_Site::base(), JSON_UNESCAPED_SLASHES);
        echo '<script>if("serviceWorker" in navigator&&(location.protocol==="https:"||location.hostname==="localhost")){addEventListener("load",function(){navigator.serviceWorker.register(' . $sw . ',{scope:' . $scope . '}).catch(function(){})})}</script>' . "\n";
    }
}
