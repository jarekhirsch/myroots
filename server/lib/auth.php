<?php
/**
 * MyRoots Server: accounts and sessions.
 *
 * Roles: "admin" (everything, including accounts and server settings),
 * "editor" (edits the trees and publishes the family website), "family"
 * (only sees the family website when it is limited to logged-in people).
 * Requests that change something need the session token (header X-WP-Nonce,
 * the name the app already uses for WordPress).
 */

declare(strict_types=1);

final class MR_Auth
{
    const ROLES = ['admin', 'editor', 'family'];
    private static ?array $user = null;
    private static bool $started = false;

    public static function start(): void
    {
        if (self::$started) return;
        self::$started = true;
        session_name('MYROOTS');
        session_set_cookie_params([
            'lifetime' => 0,
            'path' => mr_base(),
            'secure' => mr_is_https(),
            'httponly' => true,
            'samesite' => 'Lax',
        ]);
        // sessions next to the data, so a shared server's cleanup does not log people out early
        $dir = MR_DATA . '/sessions';
        if (!is_dir($dir)) @mkdir($dir, 0770, true);
        if (is_dir($dir) && is_writable($dir)) {
            MR_Db::protect($dir);
            session_save_path($dir);
            ini_set('session.gc_maxlifetime', (string) (14 * 86400));
        }
        session_start();
        if (empty($_SESSION['csrf'])) $_SESSION['csrf'] = mr_random(32);
        // "remember me": a long-lived session cookie
        if (!empty($_SESSION['remember']) && empty($_SESSION['cookie_set'])) {
            setcookie(session_name(), session_id(), ['expires' => time() + 14 * 86400, 'path' => mr_base(), 'secure' => mr_is_https(), 'httponly' => true, 'samesite' => 'Lax']);
            $_SESSION['cookie_set'] = 1;
        }
    }

    public static function user(): ?array
    {
        if (self::$user !== null) return self::$user ?: null;
        self::start();
        $id = (int) ($_SESSION['uid'] ?? 0);
        $u = null;
        if ($id) {
            $st = MR_Db::pdo()->prepare('SELECT * FROM users WHERE id = ?');
            $st->execute([$id]);
            $u = $st->fetch() ?: null;
            // a changed password ends the other sessions
            if ($u && ($_SESSION['pv'] ?? '') !== substr(hash('sha256', $u['pass']), 0, 16)) $u = null;
        }
        self::$user = $u ?: [];
        return $u;
    }

    public static function role(): string
    {
        return (string) (self::user()['role'] ?? '');
    }

    public static function can_edit(): bool
    {
        return in_array(self::role(), ['admin', 'editor'], true);
    }

    public static function is_admin(): bool
    {
        return self::role() === 'admin';
    }

    public static function csrf(): string
    {
        self::start();
        return (string) $_SESSION['csrf'];
    }

    public static function check_csrf(): bool
    {
        $t = (string) ($_SERVER['HTTP_X_WP_NONCE'] ?? ($_GET['_wpnonce'] ?? ($_POST['_token'] ?? '')));
        return $t !== '' && hash_equals(self::csrf(), $t);
    }

    /* ---------------- logging in ---------------- */

    private static function too_many(string $kind, int $max, int $window): bool
    {
        $db = MR_Db::pdo();
        $db->prepare('DELETE FROM attempts WHERE t < ?')->execute([time() - 86400]);
        $st = $db->prepare('SELECT COUNT(*) FROM attempts WHERE ip = ? AND kind = ? AND t > ?');
        $st->execute([mr_ip(), $kind, time() - $window]);
        return (int) $st->fetchColumn() >= $max;
    }

    public static function note_attempt(string $kind): void
    {
        MR_Db::pdo()->prepare('INSERT INTO attempts (ip, kind, t) VALUES (?, ?, ?)')->execute([mr_ip(), $kind, time()]);
    }

    public static function limited(string $kind, int $max, int $window): bool
    {
        return self::too_many($kind, $max, $window);
    }

    /** @return string '' on success, else an error message */
    public static function login(string $login, string $pass, bool $remember): string
    {
        if (self::too_many('login', 8, 900)) return mr_t('Too many attempts. Try again in a quarter of an hour.');
        $st = MR_Db::pdo()->prepare('SELECT * FROM users WHERE login = ?');
        $st->execute([trim($login)]);
        $u = $st->fetch();
        if (!$u || !password_verify($pass, $u['pass'])) {
            self::note_attempt('login');
            usleep(400000);
            return mr_t('Wrong login or password.');
        }
        if (password_needs_rehash($u['pass'], PASSWORD_DEFAULT)) {
            $u['pass'] = password_hash($pass, PASSWORD_DEFAULT);
            MR_Db::pdo()->prepare('UPDATE users SET pass = ? WHERE id = ?')->execute([$u['pass'], $u['id']]);
        }
        self::start();
        session_regenerate_id(true);
        $_SESSION['uid'] = (int) $u['id'];
        $_SESSION['pv'] = substr(hash('sha256', $u['pass']), 0, 16);
        $_SESSION['csrf'] = mr_random(32);
        $_SESSION['remember'] = $remember ? 1 : 0;
        unset($_SESSION['cookie_set']);
        MR_Db::pdo()->prepare('UPDATE users SET last_login = ? WHERE id = ?')->execute([time(), $u['id']]);
        self::$user = null;
        return '';
    }

    public static function logout(): void
    {
        self::start();
        $_SESSION = [];
        setcookie(session_name(), '', ['expires' => time() - 3600, 'path' => mr_base()]);
        session_destroy();
        self::$user = [];
    }

    /* ---------------- accounts ---------------- */

    public static function count(): int
    {
        try {
            return (int) MR_Db::pdo()->query('SELECT COUNT(*) FROM users')->fetchColumn();
        } catch (Throwable $e) {
            return 0;
        }
    }

    public static function all(): array
    {
        return MR_Db::pdo()->query('SELECT id, login, name, email, role, created, last_login FROM users ORDER BY role, login')->fetchAll();
    }

    /** @return string '' on success, else an error message */
    public static function create(string $login, string $name, string $email, string $pass, string $role): string
    {
        $login = trim($login);
        if (!preg_match('/^[A-Za-z0-9._@-]{3,60}$/', $login)) return mr_t('The login may have 3 to 60 letters, digits and . _ @ -');
        if (strlen($pass) < 8) return mr_t('The password must have at least 8 characters.');
        if (!in_array($role, self::ROLES, true)) $role = 'editor';
        if ($email !== '' && !filter_var($email, FILTER_VALIDATE_EMAIL)) return mr_t('The e-mail address is not valid.');
        try {
            MR_Db::pdo()->prepare('INSERT INTO users (login, name, email, pass, role, created) VALUES (?, ?, ?, ?, ?, ?)')
                ->execute([$login, mr_text($name, 100), $email, password_hash($pass, PASSWORD_DEFAULT), $role, time()]);
        } catch (PDOException $e) {
            return mr_t('This login is already taken.');
        }
        return '';
    }

    public static function set_password(int $id, string $pass): string
    {
        if (strlen($pass) < 8) return mr_t('The password must have at least 8 characters.');
        $hash = password_hash($pass, PASSWORD_DEFAULT);
        MR_Db::pdo()->prepare('UPDATE users SET pass = ? WHERE id = ?')->execute([$hash, $id]);
        if ($id === (int) ($_SESSION['uid'] ?? 0)) $_SESSION['pv'] = substr(hash('sha256', $hash), 0, 16);
        self::$user = null;
        return '';
    }

    public static function set_role(int $id, string $role): void
    {
        if (!in_array($role, self::ROLES, true)) return;
        MR_Db::pdo()->prepare('UPDATE users SET role = ? WHERE id = ?')->execute([$role, $id]);
    }

    public static function delete(int $id): void
    {
        MR_Db::pdo()->prepare('DELETE FROM users WHERE id = ?')->execute([$id]);
    }

    public static function admins(): int
    {
        return (int) MR_Db::pdo()->query("SELECT COUNT(*) FROM users WHERE role = 'admin'")->fetchColumn();
    }

    /** The tree this editor opened last. */
    public static function tree(): string
    {
        return (string) (self::user()['tree'] ?? '');
    }

    public static function remember_tree(string $id): void
    {
        $u = self::user();
        if (!$u) return;
        MR_Db::pdo()->prepare('UPDATE users SET tree = ? WHERE id = ?')->execute([$id, $u['id']]);
        self::$user = null;
    }
}
