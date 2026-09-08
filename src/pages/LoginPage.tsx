import { useState } from 'react';
import { Link } from 'react-router-dom';

export default function LoginPage() {
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [role, setRole] = useState<'fan' | 'author'>('fan');

  return (
    <div className="min-h-screen bg-ink text-paper flex items-center justify-center px-6">
      <div className="w-full max-w-md">
        {/* Logo */}
        <Link to="/" className="flex items-center gap-2.5 mb-12 justify-center">
          <div className="w-7 h-7 border border-copper flex items-center justify-center">
            <div className="w-2 h-2 bg-copper" />
          </div>
          <span className="font-display font-semibold text-lg tracking-tight">ЧИСТОВИК</span>
        </Link>

        {/* Form card */}
        <div className="surface p-8">
          <h1 className="font-display text-2xl font-semibold mb-2">
            {mode === 'login' ? 'Вход' : 'Регистрация'}
          </h1>
          <p className="text-sm text-paper-muted mb-8">
            {mode === 'login' ? 'Войдите в свой аккаунт' : 'Создайте аккаунт за минуту'}
          </p>

          {mode === 'register' && (
            <div className="flex gap-2 mb-6">
              <button
                onClick={() => setRole('fan')}
                className={`flex-1 py-2.5 text-sm rounded transition ${role === 'fan' ? 'bg-copper text-ink font-medium' : 'text-paper-muted border border-ink-border hover:border-ink-border-strong'}`}
              >
                Слушатель
              </button>
              <button
                onClick={() => setRole('author')}
                className={`flex-1 py-2.5 text-sm rounded transition ${role === 'author' ? 'bg-copper text-ink font-medium' : 'text-paper-muted border border-ink-border hover:border-ink-border-strong'}`}
              >
                Автор
              </button>
            </div>
          )}

          <form className="space-y-4" onSubmit={e => { e.preventDefault(); window.location.href = role === 'author' ? '/author/dashboard' : '/user/dashboard'; }}>
            {mode === 'register' && (
              <div>
                <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Имя</label>
                <input type="text" placeholder="Как вас зовут?" className="input w-full px-4 py-3 rounded text-sm" />
              </div>
            )}
            <div>
              <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Email</label>
              <input type="email" placeholder="you@example.com" className="input w-full px-4 py-3 rounded text-sm" />
            </div>
            <div>
              <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Пароль</label>
              <input type="password" placeholder="Минимум 8 символов" className="input w-full px-4 py-3 rounded text-sm" />
            </div>

            {mode === 'login' && (
              <div className="flex items-center justify-between text-sm">
                <label className="flex items-center gap-2 text-paper-muted cursor-pointer">
                  <input type="checkbox" className="accent-copper" />
                  Запомнить
                </label>
                <a href="#" className="text-copper hover:underline">Забыли пароль?</a>
              </div>
            )}

            <button type="submit" className="btn-primary w-full py-3 rounded text-sm mt-2">
              {mode === 'login' ? 'Войти' : 'Создать аккаунт'}
            </button>
          </form>

          <div className="divider my-6" />

          <div className="space-y-2">
            <button className="btn-ghost w-full py-2.5 rounded text-sm">Войти через Telegram</button>
            <button className="btn-ghost w-full py-2.5 rounded text-sm">Войти через VK ID</button>
          </div>
        </div>

        <p className="text-center text-sm text-paper-muted mt-6">
          {mode === 'login' ? (
            <>Нет аккаунта? <button onClick={() => setMode('register')} className="text-copper hover:underline">Зарегистрироваться</button></>
          ) : (
            <>Уже есть аккаунт? <button onClick={() => setMode('login')} className="text-copper hover:underline">Войти</button></>
          )}
        </p>
      </div>
    </div>
  );
}
