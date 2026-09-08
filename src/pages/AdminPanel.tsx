import { useState, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { LayoutDashboard, Users, Music, DollarSign, AlertTriangle, Settings } from 'lucide-react';

type Tab = 'overview' | 'users' | 'authors' | 'transactions' | 'moderation' | 'settings';

export default function AdminPanel() {
  const [activeTab, setActiveTab] = useState<Tab>('overview');

  const platformStats = [
    { label: 'Пользователи', value: '12,458', change: '+24%' },
    { label: 'Авторы', value: '186', change: '+3' },
    { label: 'MRR', value: '2.4M ₽', change: '+18%' },
    { label: 'Комиссия', value: '192K ₽', change: '+12%' },
  ];

  const tabs: { id: Tab; label: string; icon: ReactNode }[] = [
    { id: 'overview', label: 'Обзор', icon: <LayoutDashboard className="w-4 h-4" /> },
    { id: 'users', label: 'Пользователи', icon: <Users className="w-4 h-4" /> },
    { id: 'authors', label: 'Авторы', icon: <Music className="w-4 h-4" /> },
    { id: 'transactions', label: 'Транзакции', icon: <DollarSign className="w-4 h-4" /> },
    { id: 'moderation', label: 'Модерация', icon: <AlertTriangle className="w-4 h-4" /> },
    { id: 'settings', label: 'Настройки', icon: <Settings className="w-4 h-4" /> },
  ];

  return (
    <div className="min-h-screen bg-ink text-paper flex">
      {/* Sidebar */}
      <aside className="w-60 border-r border-ink-border flex flex-col fixed h-full bg-ink">
        <div className="p-4 border-b border-ink-border">
          <Link to="/" className="flex items-center gap-2.5">
            <div className="w-7 h-7 border border-copper flex items-center justify-center">
              <div className="w-2 h-2 bg-copper" />
            </div>
            <span className="font-display font-semibold tracking-tight">ЧИСТОВИК</span>
          </Link>
          <div className="mt-2">
            <span className="tag tag-copper text-[10px]">Admin</span>
          </div>
        </div>

        <nav className="flex-1 p-3 space-y-0.5">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded text-sm transition ${activeTab === tab.id ? 'bg-ink-surface text-paper' : 'text-paper-muted hover:text-paper hover:bg-ink-elevated'}`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </nav>
      </aside>

      {/* Main */}
      <main className="flex-1 ml-60">
        <div className="h-14 border-b border-ink-border flex items-center px-6">
          <div className="font-mono text-xs text-paper-dim uppercase tracking-wider">
            {tabs.find(t => t.id === activeTab)?.label}
          </div>
        </div>

        <div className="p-6">
          {activeTab === 'overview' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Панель администратора</h1>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-px bg-ink-border border border-ink-border mb-8">
                {platformStats.map((stat, i) => (
                  <div key={i} className="bg-ink p-5">
                    <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-2">{stat.label}</div>
                    <div className="flex items-baseline gap-2">
                      <span className="font-display text-2xl font-semibold num-display">{stat.value}</span>
                      <span className="text-xs text-success font-mono">{stat.change}</span>
                    </div>
                  </div>
                ))}
              </div>

              <div className="grid md:grid-cols-2 gap-6">
                <div className="surface">
                  <div className="px-5 py-4 border-b border-ink-border flex items-center justify-between">
                    <h2 className="font-medium">Ожидают модерации</h2>
                    <span className="tag tag-copper text-[10px]">3</span>
                  </div>
                  <div className="divide-y divide-ink-border">
                    {[
                      { name: 'Елена Тихонова', type: 'Подкасты', date: 'Сегодня' },
                      { name: 'Роман Козлов', type: 'Электронная музыка', date: 'Вчера' },
                      { name: 'Ольга Новикова', type: 'Мастер-классы', date: '2 дня назад' },
                    ].map((a, i) => (
                      <div key={i} className="px-5 py-3 flex items-center justify-between">
                        <div>
                          <div className="text-sm font-medium">{a.name}</div>
                          <div className="text-xs text-paper-muted">{a.type} · {a.date}</div>
                        </div>
                        <div className="flex gap-1">
                          <button className="px-2 py-1 rounded text-xs border border-success/30 text-success hover:bg-success/10">✓</button>
                          <button className="px-2 py-1 rounded text-xs border border-danger/30 text-danger hover:bg-danger/10">×</button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="surface">
                  <div className="px-5 py-4 border-b border-ink-border flex items-center justify-between">
                    <h2 className="font-medium">Жалобы</h2>
                    <span className="tag text-[10px] border-warning/30 text-warning">2</span>
                  </div>
                  <div className="divide-y divide-ink-border">
                    {[
                      { content: 'Трек "Тёмная сторона"', reason: 'Нарушение АП', date: '1 час назад' },
                      { content: 'Видео "Studio Vlog"', reason: 'Неприемлемый контент', date: '3 часа назад' },
                    ].map((c, i) => (
                      <div key={i} className="px-5 py-3 flex items-center justify-between">
                        <div>
                          <div className="text-sm font-medium">{c.content}</div>
                          <div className="text-xs text-paper-muted">{c.reason} · {c.date}</div>
                        </div>
                        <button className="text-xs text-copper hover:underline">Рассмотреть</button>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'users' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Пользователи</h1>
              <div className="surface">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-ink-border">
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Email</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Имя</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Подписки</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Статус</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-ink-border">
                    {[
                      { email: 'ivan@mail.ru', name: 'Иван Петров', subs: 3, active: true },
                      { email: 'maria@gmail.com', name: 'Мария Сидорова', subs: 1, active: true },
                      { email: 'artem@yandex.ru', name: 'Артём Козлов', subs: 5, active: true },
                      { email: 'natasha@mail.ru', name: 'Наталья Иванова', subs: 0, active: false },
                    ].map((u, i) => (
                      <tr key={i} className="hover:bg-ink-elevated transition">
                        <td className="px-5 py-3 text-sm font-mono">{u.email}</td>
                        <td className="px-5 py-3 text-sm">{u.name}</td>
                        <td className="px-5 py-3 text-sm font-mono num-display">{u.subs}</td>
                        <td className="px-5 py-3">
                          <span className={`tag text-[10px] ${u.active ? 'border-success/30 text-success' : 'border-danger/30 text-danger'}`}>
                            {u.active ? 'Active' : 'Banned'}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {activeTab === 'authors' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Авторы</h1>
              <div className="surface">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-ink-border">
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Имя</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Специализация</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Подписчики</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Верификация</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-ink-border">
                    {[
                      { name: 'Алексей Морозов', spec: 'Электронная музыка', subs: 1247, verified: true },
                      { name: 'Мария Светлова', spec: 'Видео-арт', subs: 892, verified: true },
                      { name: 'Дмитрий Волков', spec: 'Инди-рок', subs: 2103, verified: true },
                      { name: 'Анна Козлова', spec: 'Мастер-классы', subs: 567, verified: false },
                    ].map((a, i) => (
                      <tr key={i} className="hover:bg-ink-elevated transition">
                        <td className="px-5 py-3 text-sm font-medium">{a.name}</td>
                        <td className="px-5 py-3 text-sm text-paper-muted">{a.spec}</td>
                        <td className="px-5 py-3 text-sm font-mono num-display">{a.subs.toLocaleString()}</td>
                        <td className="px-5 py-3">
                          <span className={`tag text-[10px] ${a.verified ? 'border-success/30 text-success' : 'border-warning/30 text-warning'}`}>
                            {a.verified ? 'Verified' : 'Pending'}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {activeTab === 'transactions' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Транзакции</h1>
              <div className="surface">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-ink-border">
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">ID</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Пользователь</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Автор</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Сумма</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Комиссия</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Статус</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-ink-border">
                    {[
                      { id: 'TX-8842', user: 'Иван П.', author: 'А. Морозов', amount: 349, commission: 28, status: 'succeeded' },
                      { id: 'TX-8841', user: 'Мария С.', author: 'М. Светлова', amount: 249, commission: 20, status: 'succeeded' },
                      { id: 'TX-8840', user: 'Артём К.', author: 'Д. Волков', amount: 199, commission: 16, status: 'succeeded' },
                      { id: 'TX-8839', user: 'Денис В.', author: 'А. Морозов', amount: 199, commission: 16, status: 'refunded' },
                    ].map((t, i) => (
                      <tr key={i} className="hover:bg-ink-elevated transition">
                        <td className="px-5 py-3 text-sm font-mono">{t.id}</td>
                        <td className="px-5 py-3 text-sm">{t.user}</td>
                        <td className="px-5 py-3 text-sm">{t.author}</td>
                        <td className="px-5 py-3 text-sm font-mono num-display">{t.amount} ₽</td>
                        <td className="px-5 py-3 text-sm font-mono text-success num-display">{t.commission} ₽</td>
                        <td className="px-5 py-3">
                          <span className={`tag text-[10px] ${t.status === 'succeeded' ? 'border-success/30 text-success' : 'border-danger/30 text-danger'}`}>
                            {t.status}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {activeTab === 'moderation' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Модерация</h1>
              <div className="space-y-4">
                {[
                  { type: 'copyright', content: 'Трек "Тёмная сторона"', reason: 'Нарушение авторских прав', urgent: true },
                  { type: 'content', content: 'Видео "Studio Vlog #5"', reason: 'Неприемлемый контент', urgent: false },
                ].map((report, i) => (
                  <div key={i} className="surface p-6">
                    <div className="flex items-start justify-between">
                      <div>
                        <div className="flex items-center gap-2 mb-2">
                          <span className={`tag text-[10px] ${report.type === 'copyright' ? 'border-danger/30 text-danger' : 'border-warning/30 text-warning'}`}>
                            {report.type}
                          </span>
                          {report.urgent && <span className="tag tag-copper text-[10px]">Urgent</span>}
                        </div>
                        <h3 className="font-medium mb-1">{report.content}</h3>
                        <p className="text-sm text-paper-muted">{report.reason}</p>
                      </div>
                      <div className="flex gap-2">
                        <button className="px-3 py-1.5 rounded text-xs border border-success/30 text-success hover:bg-success/10">Подтвердить</button>
                        <button className="px-3 py-1.5 rounded text-xs border border-danger/30 text-danger hover:bg-danger/10">Отклонить</button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'settings' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Настройки платформы</h1>
              <div className="surface p-6 space-y-6 max-w-2xl">
                <div>
                  <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Комиссия (%)</label>
                  <input type="number" defaultValue={8} className="input w-full px-4 py-3 rounded text-sm font-mono" />
                </div>
                <div>
                  <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Макс. сессий на пользователя</label>
                  <input type="number" defaultValue={2} className="input w-full px-4 py-3 rounded text-sm font-mono" />
                </div>
                <button className="btn-primary px-6 py-2.5 rounded text-sm">Сохранить</button>
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
