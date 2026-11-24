<?php
/**
 * MyRoots Server: the API of the editor (the same as in the WordPress plugin,
 * so the app uses one backend for both) and the suggestion form of the
 * family website.
 */

declare(strict_types=1);

final class MR_Api
{
    public static function dispatch(string $route): void
    {
        $m = $_SERVER['REQUEST_METHOD'] ?? 'GET';
        mr_nocache();

        // public: suggestions from the family website
        if ($route === 'suggest' && $m === 'POST') {
            self::pick_tree();
            self::suggest();
        }
        if ($route === 'nonce') {
            if (!MR_Auth::can_edit()) mr_json(['nonce' => ''], 403);
            mr_json(['nonce' => MR_Auth::csrf()]);
        }

        if (!MR_Auth::user()) mr_error('rest_cookie_invalid_nonce', 'not logged in', 403);
        if (!MR_Auth::can_edit()) mr_error('rest_forbidden', 'not allowed', 403);
        if ($m !== 'GET' && !MR_Auth::check_csrf()) mr_error('rest_cookie_invalid_nonce', 'session expired', 403);
        self::pick_tree();

        $tid = '([a-z0-9]+)';
        switch (true) {
            case $route === 'records' && $m === 'GET':
                MR_Store::stream_all();
            case $route === 'records' && $m === 'POST':
                $b = mr_body_json();
                mr_json(['saved' => MR_Store::apply((array) ($b['records'] ?? []), array_map('intval', (array) ($b['deleted_media'] ?? [])))]);
            case $route === 'reset' && $m === 'POST':
                mr_json(['gen' => MR_Store::reset()]);
            case (bool) preg_match('#^media/(\d+)$#', $route, $mm):
                $m === 'POST' ? self::media_post((int) $mm[1]) : self::media_get((int) $mm[1]);
            case $route === 'trees' && $m === 'GET':
                mr_json(['current' => MR_Trees::for_user(), 'trees' => MR_Trees::export()]);
            case $route === 'trees' && $m === 'POST':
                if (count(MR_Trees::all()) >= 50) mr_error('myroots_tree', 'too many family trees', 400);
                mr_json(MR_Trees::create((string) (mr_body_json()['name'] ?? '')));
            case (bool) preg_match("#^trees/$tid$#", $route, $mm) && $m === 'POST':
                if (!MR_Trees::rename($mm[1], (string) (mr_body_json()['name'] ?? ''))) mr_error('myroots_tree', 'no such family tree', 404);
                mr_json(['ok' => true]);
            case (bool) preg_match("#^trees/$tid/delete$#", $route, $mm) && $m === 'POST':
                if ($mm[1] === MR_Trees::for_user()) mr_error('myroots_tree', 'open another tree first', 400);
                if (!MR_Trees::delete($mm[1])) mr_error('myroots_tree', 'no such family tree', 404);
                mr_json(['ok' => true]);
            case (bool) preg_match("#^trees/$tid/open$#", $route, $mm) && $m === 'POST':
                if (!MR_Trees::exists($mm[1])) mr_error('myroots_tree', 'no such family tree', 404);
                MR_Auth::remember_tree($mm[1]);
                mr_json(['ok' => true]);
            case $route === 'legacy':
                mr_json(['persons' => [], 'families' => [], 'pets' => [], 'sources' => []]);
            case $route === 'site' && $m === 'GET':
                mr_json(self::site_state());
            case $route === 'site/options' && $m === 'POST':
                $b = mr_body_json();
                if (isset($b['auto'])) MR_Trees::update_opt('auto_publish', (bool) $b['auto']);
                if (!empty($b['weather_decided'])) MR_Trees::update_opt('weather_decided', 1);
                mr_json(self::site_state());
            case $route === 'site/files' && $m === 'GET':
                // only photos that are really on the disk: a missing file
                // (interrupted upload, files copied by hand) is sent again
                $sigs = (array) (MR_Site::state()['sigs'] ?? []);
                $media = MR_Store::dir('site') . '/media/';
                foreach ($sigs as $id => $sig) {
                    if (!glob($media . (int) $id . '.*') && !glob($media . (int) $id . '_t.*')) unset($sigs[$id]);
                }
                mr_json(['sigs' => (object) $sigs]);
            case $route === 'site/begin' && $m === 'POST':
                $token = mr_random(16);
                MR_Opt::set_temp('publish' . MR_Trees::suffix(), $token, 3600);
                mr_json(['token' => $token]);
            case $route === 'site/file' && $m === 'POST':
                self::site_file();
            case $route === 'site/finish' && $m === 'POST':
                self::site_finish();
            case $route === 'suggestions' && $m === 'GET':
                $box = (array) MR_Trees::get_opt('inbox', []);
                mr_json(['codes' => array_values($box), 'ids' => array_keys($box)]);
            case $route === 'suggestions/done' && $m === 'POST':
                $box = (array) MR_Trees::get_opt('inbox', []);
                foreach ((array) (mr_body_json()['ids'] ?? []) as $id) unset($box[(string) $id]);
                MR_Trees::update_opt('inbox', $box);
                mr_json(['ok' => true]);
        }
        mr_error('rest_no_route', 'no such address', 404);
    }

    private static function pick_tree(): void
    {
        $tree = (string) ($_GET['tree'] ?? '');
        if ($tree === '') MR_Trees::set(MR_Trees::main());
        elseif (!MR_Trees::set($tree)) mr_error('myroots_tree', 'no such family tree', 404);
    }

    private static function site_state(): array
    {
        $st = MR_Site::state();
        return [
            'url' => MR_Site::base(),
            'published' => (int) ($st['published'] ?? 0) * 1000,
            'auto' => (bool) MR_Trees::get_opt('auto_publish', true),
            'persons' => (int) ($st['persons'] ?? 0),
            'media' => count((array) ($st['sigs'] ?? [])),
        ];
    }

    private static function media_post(int $id): void
    {
        $thumb = (int) ($_GET['thumb'] ?? 0) === 1;
        $err = MR_Store::write_chunk(MR_Store::media_path($id, $thumb), (string) file_get_contents('php://input'), (int) ($_GET['offset'] ?? 0), (int) ($_GET['final'] ?? 0) === 1);
        if ($err !== '') mr_error('myroots_upload', $err, $err === 'upload pieces out of order' ? 409 : 500);
        mr_json(['ok' => true]);
    }

    private static function media_get(int $id): void
    {
        $thumb = (int) ($_GET['t'] ?? 0) === 1;
        $strict = (int) ($_GET['strict'] ?? 0) === 1;
        $path = MR_Store::media_path($id, $thumb);
        if ($thumb && !is_file($path) && !$strict) {
            $path = MR_Store::media_path($id, false);
            $thumb = false;
        }
        $mime = 'image/jpeg';
        if (!$thumb) {
            $rec = MR_Store::get('media', $id);
            $mime = (string) ($rec['data']['mime'] ?? 'application/octet-stream');
            if (!preg_match('#^[a-z]+/[a-z0-9.+-]+$#i', $mime) || preg_match('#html|svg|xml#i', $mime)) $mime = 'application/octet-stream';
        }
        header("Content-Security-Policy: default-src 'none'; sandbox");
        MR_Store::send_file($path, $mime, true);
    }

    private static function check_token(string $token): bool
    {
        $t = MR_Opt::get_temp('publish' . MR_Trees::suffix());
        return is_string($t) && $token !== '' && hash_equals($t, $token);
    }

    private static function site_file(): void
    {
        if (!self::check_token((string) ($_GET['token'] ?? ''))) mr_error('myroots_token', 'publishing session expired', 409);
        $path = (string) ($_GET['path'] ?? '');
        $offset = (int) ($_GET['offset'] ?? 0);
        $dir = MR_Store::dir('site');
        if ($path === 'data.js') {
            $dest = $dir . '/data.next.js';
        } elseif (preg_match('#^media/((\d+(?:_t)?)\.[a-z0-9]{2,5})$#', $path, $m)) {
            if (!is_dir($dir . '/media')) @mkdir($dir . '/media', 0775, true);
            $dest = $dir . '/media/' . $m[1];
            if ($offset === 0) {
                foreach ((array) glob($dir . '/media/' . $m[2] . '.*') as $old) {
                    if (is_string($old) && $old !== $dest && substr($old, -5) !== '.part') @unlink($old);
                }
            }
        } else {
            mr_error('myroots_path', 'invalid file name', 400);
        }
        $err = MR_Store::write_chunk($dest, (string) file_get_contents('php://input'), $offset, (int) ($_GET['final'] ?? 0) === 1);
        if ($err !== '') mr_error('myroots_upload', $err, 500);
        mr_json(['ok' => true]);
    }

    private static function site_finish(): void
    {
        $b = mr_body_json();
        if (!self::check_token((string) ($b['token'] ?? ''))) mr_error('myroots_token', 'publishing session expired', 409);
        $sigs = [];
        foreach ((array) ($b['sigs'] ?? []) as $id => $sig) $sigs[(int) $id] = substr(mr_text((string) $sig, 80), 0, 80);
        $err = MR_Site::finish($sigs);
        if ($err !== '') mr_error('myroots_publish', $err, 500);
        MR_Opt::delete('tmp_publish' . MR_Trees::suffix());
        mr_json(self::site_state());
    }

    private static function suggest(): void
    {
        $b = mr_body_json();
        $code = (string) ($b['code'] ?? '');
        if (strlen($code) > 20000 || !preg_match('#^MYROOTS-SUGGESTION:[A-Za-z0-9+/=]+:END$#', $code)) mr_error('myroots_suggest', 'invalid suggestion', 400);
        if (!MR_Site::visible()) mr_error('myroots_suggest', 'not available', 403);
        if (MR_Auth::limited('suggest', 10, 3600)) mr_error('myroots_suggest', 'too many suggestions, try later', 429);
        MR_Auth::note_attempt('suggest');
        $box = (array) MR_Trees::get_opt('inbox', []);
        if (count($box) >= 300) mr_error('myroots_suggest', 'the inbox is full', 429);
        $box[mr_random(10)] = $code;
        MR_Trees::update_opt('inbox', $box);

        $to = (string) MR_Opt::get('notify_email', '');
        if ($to !== '' && MR_Opt::get('notify', true)) {
            $name = mr_text((string) ($b['person_name'] ?? ''), 120);
            $who = mr_text((string) ($b['author'] ?? ''), 120);
            $msg = mb_substr(strip_tags((string) ($b['message'] ?? '')), 0, 4000);
            $subject = '=?UTF-8?B?' . base64_encode(sprintf(mr_t('[MyRoots] New correction suggestion: %s'), $name)) . '?=';
            $body = $msg . "\n\n" . ($who !== '' ? mr_t('From:') . ' ' . $who . "\n" : '') . "\n" . mr_abs('admin?tree=' . MR_Trees::current() . '#/suggestions') . "\n";
            $host = preg_replace('/[^A-Za-z0-9.\-]/', '', (string) ($_SERVER['HTTP_HOST'] ?? 'localhost'));
            @mail($to, $subject, $body, "Content-Type: text/plain; charset=UTF-8\r\nFrom: MyRoots <no-reply@" . preg_replace('/:\d+$/', '', $host) . ">\r\n");
        }
        mr_json(['ok' => true]);
    }
}
