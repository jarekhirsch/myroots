<?php
/**
 * MyRoots Server: pages of the server itself (first start, logging in,
 * server settings and accounts). Plain HTML in the look of the app.
 */

declare(strict_types=1);

final class MR_Pages
{
    public static function shell(string $title, string $body): void
    {
        $css = <<<CSS
:root{--bg:#f6f8f6;--card:#fff;--ink:#1c2420;--muted:#5d6b64;--line:#e2e8e4;--accent:#1d5a44;--accent-ink:#fff;--danger:#b3261e;--ok:#1d5a44;color-scheme:light}
@media (prefers-color-scheme:dark){:root{--bg:#121614;--card:#1a201d;--ink:#e8eee9;--muted:#9aa8a1;--line:#2b3430;--accent:#5fb48e;--accent-ink:#0e1a14;--danger:#ff8a80;color-scheme:dark}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--ink);font:16px/1.55 system-ui,-apple-system,"Segoe UI",Roboto,sans-serif}
main{max-width:880px;margin:0 auto;padding:32px 16px 64px}
.card{background:var(--card);border:1px solid var(--line);border-radius:18px;padding:24px;margin:0 0 18px}
.card.narrow{max-width:420px;margin:8vh auto 0}.center{text-align:center;max-width:520px;margin:12vh auto 0}
h1{font-size:1.5rem;margin:.2em 0 .6em;line-height:1.25}h2{font-size:1.15rem;margin:0 0 .8em}
p{margin:.4em 0 .9em}.muted{color:var(--muted)}.small{font-size:.88rem}
label{display:block;margin:0 0 12px}label>span{display:block;font-size:.88rem;color:var(--muted);margin:0 0 4px}
input[type=text],input[type=password],input[type=email],select{width:100%;padding:10px 12px;border:1px solid var(--line);border-radius:10px;background:var(--bg);color:var(--ink);font:inherit}
input:focus,select:focus,.btn:focus-visible{outline:2px solid var(--accent);outline-offset:1px}
label.check{display:flex;gap:8px;align-items:flex-start}label.check input{margin-top:4px}
.btn{display:inline-flex;align-items:center;gap:6px;padding:10px 16px;border-radius:10px;border:1px solid var(--accent);background:var(--accent);color:var(--accent-ink);font:inherit;font-weight:600;text-decoration:none;cursor:pointer}
.btn.ghost{background:transparent;color:var(--ink);border-color:var(--line)}.btn.danger{background:transparent;color:var(--danger);border-color:var(--line)}.btn.sm{padding:6px 10px;font-size:.88rem}
.row{display:flex;gap:10px;flex-wrap:wrap;align-items:center}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0 16px}
.msg{padding:10px 14px;border-radius:10px;margin:0 0 16px;border:1px solid var(--line)}.msg.err{color:var(--danger)}.msg.ok{color:var(--ok)}
table{width:100%;border-collapse:collapse;font-size:.94rem}td,th{padding:8px 6px;border-bottom:1px solid var(--line);text-align:start;vertical-align:middle}th{color:var(--muted);font-weight:600;font-size:.84rem}
.table{overflow-x:auto}.top{display:flex;justify-content:space-between;align-items:center;gap:12px;flex-wrap:wrap;margin:0 0 18px}.brand{display:flex;gap:10px;align-items:center;font-weight:700;color:var(--ink);text-decoration:none}
a{color:var(--accent)}code{background:var(--bg);border:1px solid var(--line);border-radius:6px;padding:1px 6px;font-size:.88em;word-break:break-all}
CSS;
        echo "<!doctype html>\n<html lang=\"" . mr_h(mr_lang()) . "\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"robots\" content=\"noindex\"><meta name=\"theme-color\" content=\"#1d5a44\">";
        echo '<title>' . mr_h($title) . ' · MyRoots</title><link rel="icon" type="image/svg+xml" href="' . mr_h(mr_url('app/icon.svg')) . '"><style>' . $css . '</style></head><body><main>' . $body . '</main></body></html>';
    }

    private static function msg(string $err, string $ok = ''): string
    {
        if ($err !== '') return '<div class="msg err" role="alert">' . mr_h($err) . '</div>';
        if ($ok !== '') return '<div class="msg ok" role="status">' . mr_h($ok) . '</div>';
        return '';
    }

    private static function token(): string
    {
        return '<input type="hidden" name="_token" value="' . mr_h(MR_Auth::csrf()) . '">';
    }

    /** A redirect target inside the installation only. */
    private static function next(string $fallback): string
    {
        $n = (string) ($_GET['next'] ?? $_POST['next'] ?? '');
        return ($n !== '' && str_starts_with($n, mr_base()) && !str_starts_with($n, '//') && !preg_match('#[\r\n\\\\]#', $n)) ? $n : $fallback;
    }

    /* ------------------------------------------------------------------ */
    /*  First start                                                        */
    /* ------------------------------------------------------------------ */

    public static function requirements(): array
    {
        $bad = [];
        if (PHP_VERSION_ID < 80000) $bad[] = sprintf(mr_t('PHP 8.0 or newer is needed (this server has %s).'), PHP_VERSION);
        if (!extension_loaded('pdo_sqlite')) $bad[] = mr_t('The PHP extension pdo_sqlite is missing (ask the hosting company to turn it on).');
        if (!extension_loaded('mbstring')) $bad[] = mr_t('The PHP extension mbstring is missing.');
        if (!is_dir(MR_DATA) && !@mkdir(MR_DATA, 0775, true)) $bad[] = sprintf(mr_t('Cannot create the data folder %s.'), MR_DATA);
        elseif (!is_writable(MR_DATA)) $bad[] = sprintf(mr_t('The data folder %s is not writable.'), MR_DATA);
        if (!is_file(MR_ROOT . '/app/index.html')) $bad[] = mr_t('The app folder is missing (upload all files).');
        return $bad;
    }

    public static function setup(): void
    {
        $bad = self::requirements();
        if (!$bad && MR_Db::installed() && MR_Auth::count() > 0) mr_redirect(mr_url('admin'));
        $err = '';
        if (!$bad && ($_SERVER['REQUEST_METHOD'] ?? '') === 'POST') {
            MR_Db::install();
            MR_Auth::start();
            if (!MR_Auth::check_csrf()) $err = mr_t('The form has expired. Try again.');
            elseif (($_POST['pass'] ?? '') !== ($_POST['pass2'] ?? '')) $err = mr_t('The passwords are not the same.');
            else {
                $err = MR_Auth::create((string) ($_POST['login'] ?? ''), (string) ($_POST['name'] ?? ''), trim((string) ($_POST['email'] ?? '')), (string) ($_POST['pass'] ?? ''), 'admin');
                if ($err === '') {
                    MR_Opt::set('title', mr_text((string) ($_POST['title'] ?? ''), 120) ?: 'MyRoots');
                    MR_Opt::set('lang', mr_lang());
                    MR_Opt::set('notify_email', trim((string) ($_POST['email'] ?? '')));
                    MR_Opt::set('visibility', in_array($_POST['visibility'] ?? '', ['public', 'users', 'off'], true) ? $_POST['visibility'] : 'users');
                    MR_Auth::login((string) $_POST['login'], (string) $_POST['pass'], true);
                    mr_redirect(mr_url('admin'));
                }
            }
        } elseif (!$bad) {
            MR_Db::pdo();
            MR_Auth::start();
        }
        mr_nocache();
        $h = '<div class="card narrow" style="max-width:560px"><div class="brand" style="margin-bottom:10px"><img src="' . mr_h(mr_url('app/icon.svg')) . '" width="36" height="36" alt="">MyRoots</div>';
        $h .= '<h1>' . mr_h(mr_t('Install MyRoots on this server')) . '</h1>';
        if ($bad) {
            $h .= '<div class="msg err" role="alert"><ul style="margin:0;padding-inline-start:18px"><li>' . implode('</li><li>', array_map('mr_h', $bad)) . '</li></ul></div><p><a class="btn" href="' . mr_h(mr_url('setup')) . '">' . mr_h(mr_t('Check again')) . '</a></p></div>';
            self::shell(mr_t('Install'), $h);
            exit;
        }
        $h .= '<p class="muted">' . mr_h(mr_t('Your family tree on your own server: the data, photos and the family website stay here. Create the administrator account.')) . '</p>' . self::msg($err);
        $h .= '<form method="post">' . self::token();
        $h .= '<label><span>' . mr_h(mr_t('Name of the website')) . '</span><input type="text" name="title" maxlength="120" required value="' . mr_h((string) ($_POST['title'] ?? '')) . '" placeholder="' . mr_h(mr_t('e.g. The Kowalski family')) . '"></label>';
        $h .= '<div class="grid"><label><span>' . mr_h(mr_t('Login')) . '</span><input type="text" name="login" required autocomplete="username" value="' . mr_h((string) ($_POST['login'] ?? '')) . '"></label>';
        $h .= '<label><span>' . mr_h(mr_t('Your name')) . '</span><input type="text" name="name" value="' . mr_h((string) ($_POST['name'] ?? '')) . '"></label></div>';
        $h .= '<label><span>' . mr_h(mr_t('E-mail (correction suggestions from the family)')) . '</span><input type="email" name="email" value="' . mr_h((string) ($_POST['email'] ?? '')) . '"></label>';
        $h .= '<div class="grid"><label><span>' . mr_h(mr_t('Password (at least 8 characters)')) . '</span><input type="password" name="pass" required minlength="8" autocomplete="new-password"></label>';
        $h .= '<label><span>' . mr_h(mr_t('Repeat the password')) . '</span><input type="password" name="pass2" required minlength="8" autocomplete="new-password"></label></div>';
        $h .= '<label><span>' . mr_h(mr_t('Who can see the family website')) . '</span>' . self::visibility_select('users') . '</label>';
        $h .= '<p><button class="btn">' . mr_h(mr_t('Install')) . '</button></p></form>';
        if (!mr_is_https() && !in_array($_SERVER['HTTP_HOST'] ?? '', ['localhost', '127.0.0.1'], true) && !str_starts_with((string) ($_SERVER['HTTP_HOST'] ?? ''), 'localhost:')) {
            $h .= '<p class="small muted">' . mr_h(mr_t('Tip: turn on HTTPS (a free Let’s Encrypt certificate) at your hosting company. Without it passwords travel unencrypted and the website cannot be installed as an app.')) . '</p>';
        }
        $h .= '</div>';
        self::shell(mr_t('Install'), $h);
        exit;
    }

    private static function visibility_select(string $cur): string
    {
        $o = ['public' => mr_t('Everybody who has the address'), 'users' => mr_t('Only people with an account (family)'), 'off' => mr_t('Nobody but the editors (website off)')];
        $s = '<select name="visibility">';
        foreach ($o as $k => $l) $s .= '<option value="' . $k . '"' . ($k === $cur ? ' selected' : '') . '>' . mr_h($l) . '</option>';
        return $s . '</select>';
    }

    /* ------------------------------------------------------------------ */
    /*  Logging in                                                         */
    /* ------------------------------------------------------------------ */

    public static function login(): void
    {
        MR_Auth::start();
        $next = self::next(mr_url('admin'));
        if (MR_Auth::user() && ($_SERVER['REQUEST_METHOD'] ?? '') !== 'POST') mr_redirect($next);
        $err = '';
        if (($_SERVER['REQUEST_METHOD'] ?? '') === 'POST') {
            if (!MR_Auth::check_csrf()) $err = mr_t('The form has expired. Try again.');
            else {
                $err = MR_Auth::login((string) ($_POST['login'] ?? ''), (string) ($_POST['pass'] ?? ''), !empty($_POST['remember']));
                if ($err === '') {
                    // family accounts go to the website, editors to the app
                    if (!MR_Auth::can_edit() && str_starts_with($next, mr_url('admin'))) $next = mr_url('');
                    mr_redirect($next);
                }
            }
        }
        mr_nocache();
        $title = (string) MR_Opt::get('title', 'MyRoots');
        $h = '<div class="card narrow"><div class="brand" style="margin-bottom:10px"><img src="' . mr_h(mr_url('app/icon.svg')) . '" width="36" height="36" alt="">' . mr_h($title) . '</div>';
        $h .= '<h1>' . mr_h(mr_t('Log in')) . '</h1>' . self::msg($err);
        $h .= '<form method="post">' . self::token() . '<input type="hidden" name="next" value="' . mr_h($next) . '">';
        $h .= '<label><span>' . mr_h(mr_t('Login')) . '</span><input type="text" name="login" required autocomplete="username" autofocus value="' . mr_h((string) ($_POST['login'] ?? '')) . '"></label>';
        $h .= '<label><span>' . mr_h(mr_t('Password')) . '</span><input type="password" name="pass" required autocomplete="current-password"></label>';
        $h .= '<label class="check"><input type="checkbox" name="remember" value="1"> ' . mr_h(mr_t('Remember me for 14 days')) . '</label>';
        $h .= '<p><button class="btn">' . mr_h(mr_t('Log in')) . '</button></p></form>';
        $h .= '<p class="small muted">' . mr_h(mr_t('Forgot the password? Ask the administrator of this website to set a new one.')) . '</p></div>';
        self::shell(mr_t('Log in'), $h);
        exit;
    }

    public static function logout(): void
    {
        MR_Auth::logout();
        mr_redirect(mr_url('login'));
    }

    /* ------------------------------------------------------------------ */
    /*  Server settings and accounts (administrator)                       */
    /* ------------------------------------------------------------------ */

    public static function settings(): void
    {
        if (!MR_Auth::user()) mr_redirect(mr_url('login') . '?next=' . rawurlencode(mr_url('admin/settings')));
        $admin = MR_Auth::is_admin();
        $me = MR_Auth::user();
        $err = '';
        $ok = '';
        if (($_SERVER['REQUEST_METHOD'] ?? '') === 'POST') {
            if (!MR_Auth::check_csrf()) $err = mr_t('The form has expired. Try again.');
            else {
                $do = (string) ($_POST['do'] ?? '');
                if ($do === 'password') {
                    if (!password_verify((string) ($_POST['old'] ?? ''), (string) $me['pass'])) $err = mr_t('The current password is wrong.');
                    elseif (($_POST['pass'] ?? '') !== ($_POST['pass2'] ?? '')) $err = mr_t('The passwords are not the same.');
                    else $err = MR_Auth::set_password((int) $me['id'], (string) $_POST['pass']);
                    if ($err === '') $ok = mr_t('The password was changed.');
                } elseif ($admin && $do === 'site') {
                    MR_Opt::set('title', mr_text((string) ($_POST['title'] ?? ''), 120) ?: 'MyRoots');
                    $v = (string) ($_POST['visibility'] ?? 'public');
                    MR_Opt::set('visibility', in_array($v, ['public', 'users', 'off'], true) ? $v : 'public');
                    MR_Site::mirror_all();
                    MR_Opt::set('index', !empty($_POST['index']));
                    $mail = trim((string) ($_POST['notify_email'] ?? ''));
                    if ($mail !== '' && !filter_var($mail, FILTER_VALIDATE_EMAIL)) $err = mr_t('The e-mail address is not valid.');
                    else MR_Opt::set('notify_email', $mail);
                    $lang = (string) ($_POST['lang'] ?? '');
                    MR_Opt::set('lang', in_array($lang, ['pl', 'en', 'de', 'fr', 'es', 'it', 'pt', 'nl', 'sv', 'fi', 'da', 'he'], true) ? $lang : '');
                    if ($err === '') $ok = mr_t('Saved.');
                } elseif ($admin && $do === 'add') {
                    $err = MR_Auth::create((string) ($_POST['login'] ?? ''), (string) ($_POST['name'] ?? ''), trim((string) ($_POST['email'] ?? '')), (string) ($_POST['pass'] ?? ''), (string) ($_POST['role'] ?? 'family'));
                    if ($err === '') $ok = mr_t('The account was added. Give the login and the password to the person.');
                } elseif ($admin && $do === 'role') {
                    $id = (int) ($_POST['id'] ?? 0);
                    $role = (string) ($_POST['role'] ?? '');
                    if ($id === (int) $me['id'] && $role !== 'admin' && MR_Auth::admins() < 2) $err = mr_t('There must be at least one administrator.');
                    else {
                        MR_Auth::set_role($id, $role);
                        $ok = mr_t('Saved.');
                    }
                } elseif ($admin && $do === 'reset') {
                    $err = MR_Auth::set_password((int) ($_POST['id'] ?? 0), (string) ($_POST['pass'] ?? ''));
                    if ($err === '') $ok = mr_t('The password was changed.');
                } elseif ($admin && $do === 'delete') {
                    $id = (int) ($_POST['id'] ?? 0);
                    if ($id === (int) $me['id']) $err = mr_t('You cannot delete your own account.');
                    else {
                        MR_Auth::delete($id);
                        $ok = mr_t('The account was deleted.');
                    }
                }
            }
            $me = MR_Auth::user();
        }
        mr_nocache();
        $title = (string) MR_Opt::get('title', 'MyRoots');
        $h = '<div class="top"><a class="brand" href="' . mr_h(mr_url('')) . '"><img src="' . mr_h(mr_url('app/icon.svg')) . '" width="32" height="32" alt="">' . mr_h($title) . '</a><div class="row">';
        if (MR_Auth::can_edit()) $h .= '<a class="btn" href="' . mr_h(mr_url('admin')) . '">' . mr_h(mr_t('Open the app')) . '</a>';
        $h .= '<a class="btn ghost" href="' . mr_h(mr_url('')) . '">' . mr_h(mr_t('Family website')) . '</a><a class="btn ghost" href="' . mr_h(mr_url('logout')) . '">' . mr_h(mr_t('Log out')) . '</a></div></div>';
        $h .= '<h1>' . mr_h($admin ? mr_t('Server settings') : mr_t('Your account')) . '</h1>' . self::msg($err, $ok);

        if ($admin) {
            $h .= '<section class="card"><h2>' . mr_h(mr_t('Family website')) . '</h2><form method="post">' . self::token() . '<input type="hidden" name="do" value="site">';
            $h .= '<label><span>' . mr_h(mr_t('Name of the website')) . '</span><input type="text" name="title" maxlength="120" value="' . mr_h($title) . '"></label>';
            $h .= '<label><span>' . mr_h(mr_t('Who can see the family website')) . '</span>' . self::visibility_select(MR_Site::visibility()) . '</label>';
            $h .= '<label><span>' . mr_h(mr_t('E-mail for correction suggestions from the family (empty: no e-mails)')) . '</span><input type="email" name="notify_email" value="' . mr_h((string) MR_Opt::get('notify_email', '')) . '"></label>';
            $langs = ['' => mr_t('Like the browser'), 'pl' => 'Polski', 'en' => 'English', 'de' => 'Deutsch', 'fr' => 'Français', 'es' => 'Español', 'it' => 'Italiano', 'pt' => 'Português', 'nl' => 'Nederlands', 'sv' => 'Svenska', 'fi' => 'Suomi', 'da' => 'Dansk', 'he' => 'עברית'];
            $h .= '<label><span>' . mr_h(mr_t('Language of the app for the editors')) . '</span><select name="lang">';
            foreach ($langs as $k => $l) $h .= '<option value="' . $k . '"' . ((string) MR_Opt::get('lang', '') === $k ? ' selected' : '') . '>' . mr_h($l) . '</option>';
            $h .= '</select></label>';
            $h .= '<label class="check"><input type="checkbox" name="index" value="1"' . (MR_Opt::get('index', false) ? ' checked' : '') . '> ' . mr_h(mr_t('Let search engines show the family website (only when everybody can see it)')) . '</label>';
            $h .= '<p><button class="btn">' . mr_h(mr_t('Save')) . '</button></p></form>';

            $h .= '<h2 style="margin-top:22px">' . mr_h(mr_t('Family trees')) . '</h2><div class="table"><table><thead><tr><th>' . mr_h(mr_t('Tree')) . '</th><th>' . mr_h(mr_t('Address')) . '</th><th>' . mr_h(mr_t('Published')) . '</th></tr></thead><tbody>';
            $keep = MR_Trees::current();
            foreach (MR_Trees::export() as $i => $t) {
                MR_Trees::set($t['id']);
                $pub = (int) (MR_Site::state()['published'] ?? 0);
                $h .= '<tr><td>' . mr_h($t['name'] !== '' ? $t['name'] : sprintf(mr_t('Family tree %d'), $i + 1)) . '</td><td><a href="' . mr_h($t['url']) . '">' . mr_h(mr_abs(ltrim(substr($t['url'], strlen(mr_base())), '/'))) . '</a></td><td>' . ($pub ? mr_h(date('Y-m-d H:i', $pub)) : '<span class="muted">' . mr_h(mr_t('not yet')) . '</span>') . '</td></tr>';
            }
            MR_Trees::set($keep);
            $h .= '</tbody></table></div><p class="small muted">' . mr_h(mr_t('Trees are added, renamed and deleted in the app (Family trees). Each one is published separately.')) . '</p></section>';

            $roles = ['admin' => mr_t('Administrator'), 'editor' => mr_t('Editor'), 'family' => mr_t('Family (view only)')];
            $h .= '<section class="card"><h2>' . mr_h(mr_t('Accounts')) . '</h2><p class="small muted">' . mr_h(mr_t('Editors edit the trees and publish the family website. Family accounts only view the website when it is limited to people with an account. Administrators also manage these settings.')) . '</p>';
            $h .= '<div class="table"><table><thead><tr><th>' . mr_h(mr_t('Login')) . '</th><th>' . mr_h(mr_t('Name')) . '</th><th>' . mr_h(mr_t('Role')) . '</th><th>' . mr_h(mr_t('Last login')) . '</th><th></th></tr></thead><tbody>';
            foreach (MR_Auth::all() as $u) {
                $sel = '<form method="post" class="row">' . self::token() . '<input type="hidden" name="do" value="role"><input type="hidden" name="id" value="' . (int) $u['id'] . '"><select name="role" onchange="this.form.submit()" aria-label="' . mr_h(mr_t('Role')) . '">';
                foreach ($roles as $k => $l) $sel .= '<option value="' . $k . '"' . ($u['role'] === $k ? ' selected' : '') . '>' . mr_h($l) . '</option>';
                $sel .= '</select></form>';
                $act = '<details><summary class="small">' . mr_h(mr_t('More')) . '</summary><form method="post" class="row" style="margin-top:6px">' . self::token() . '<input type="hidden" name="do" value="reset"><input type="hidden" name="id" value="' . (int) $u['id'] . '"><input type="password" name="pass" minlength="8" placeholder="' . mr_h(mr_t('New password')) . '" style="max-width:180px" autocomplete="new-password"><button class="btn sm ghost">' . mr_h(mr_t('Set')) . '</button></form>';
                if ((int) $u['id'] !== (int) $me['id']) $act .= '<form method="post" style="margin-top:6px" onsubmit="return confirm(' . mr_h(json_encode(mr_t('Delete this account?'))) . ')">' . self::token() . '<input type="hidden" name="do" value="delete"><input type="hidden" name="id" value="' . (int) $u['id'] . '"><button class="btn sm danger">' . mr_h(mr_t('Delete')) . '</button></form>';
                $act .= '</details>';
                $h .= '<tr><td>' . mr_h($u['login']) . '</td><td>' . mr_h($u['name']) . '<div class="small muted">' . mr_h($u['email']) . '</div></td><td>' . $sel . '</td><td class="small">' . ($u['last_login'] ? mr_h(date('Y-m-d', (int) $u['last_login'])) : '<span class="muted">' . mr_h(mr_t('never')) . '</span>') . '</td><td>' . $act . '</td></tr>';
            }
            $h .= '</tbody></table></div>';
            $h .= '<h2 style="margin-top:22px">' . mr_h(mr_t('New account')) . '</h2><form method="post">' . self::token() . '<input type="hidden" name="do" value="add"><div class="grid">';
            $h .= '<label><span>' . mr_h(mr_t('Login')) . '</span><input type="text" name="login" required autocomplete="off"></label>';
            $h .= '<label><span>' . mr_h(mr_t('Name')) . '</span><input type="text" name="name"></label>';
            $h .= '<label><span>' . mr_h(mr_t('E-mail (optional)')) . '</span><input type="email" name="email"></label>';
            $h .= '<label><span>' . mr_h(mr_t('Password (at least 8 characters)')) . '</span><input type="password" name="pass" required minlength="8" autocomplete="new-password"></label>';
            $h .= '<label><span>' . mr_h(mr_t('Role')) . '</span><select name="role">';
            foreach ($roles as $k => $l) $h .= '<option value="' . $k . '"' . ($k === 'family' ? ' selected' : '') . '>' . mr_h($l) . '</option>';
            $h .= '</select></label></div><p><button class="btn">' . mr_h(mr_t('Add the account')) . '</button></p></form></section>';
        }

        $h .= '<section class="card"><h2>' . mr_h(mr_t('Your password')) . '</h2><form method="post">' . self::token() . '<input type="hidden" name="do" value="password"><div class="grid">';
        $h .= '<label><span>' . mr_h(mr_t('Current password')) . '</span><input type="password" name="old" required autocomplete="current-password"></label>';
        $h .= '<label><span>' . mr_h(mr_t('New password')) . '</span><input type="password" name="pass" required minlength="8" autocomplete="new-password"></label>';
        $h .= '<label><span>' . mr_h(mr_t('Repeat the password')) . '</span><input type="password" name="pass2" required minlength="8" autocomplete="new-password"></label>';
        $h .= '</div><p><button class="btn">' . mr_h(mr_t('Change the password')) . '</button></p></form></section>';

        if ($admin) {
            $h .= '<section class="card"><h2>' . mr_h(mr_t('Server')) . '</h2><p class="small">MyRoots ' . mr_h(MR_VERSION) . ' · PHP ' . mr_h(PHP_VERSION) . ' · SQLite ' . mr_h((string) MR_Db::pdo()->query('select sqlite_version()')->fetchColumn()) . '</p>';
            $h .= '<p class="small muted">' . mr_h(mr_t('Data folder:')) . ' <code>' . mr_h(MR_DATA) . '</code></p>';
            $h .= '<p class="small muted">' . mr_h(mr_t('Back up this folder regularly (it holds the database, the photos and the published websites), or make a .myroots backup in the app: Import and backup.')) . '</p>';
            $local = preg_match('#^(localhost|127\.0\.0\.1)(:\d+)?$#', (string) ($_SERVER['HTTP_HOST'] ?? ''));
            if (!mr_is_https() && !$local) $h .= '<p class="small" style="color:var(--danger)">' . mr_h(mr_t('This address does not use HTTPS. Turn it on at your hosting company: without it passwords travel unencrypted.')) . '</p>';
            $h .= '</section>';
        }
        self::shell($admin ? mr_t('Server settings') : mr_t('Your account'), $h);
        exit;
    }
}
