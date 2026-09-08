import { useState, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { BookOpen, Heart, CreditCard, Settings } from 'lucide-react';

type Tab = 'library' | 'subscriptions' | 'payments' | 'settings';

export default function UserDashboard() {
  const [activeTab, setActiveTab] = useState<Tab>('library');

  const subscriptions = [
    { id: '1', author: 'Алексей Морозов', initials: 'АМ', modules: ['Музыка', 'Видео'], price: 349, nextPayment: '15.02.2026', status: 'active' },
    { id: '2', author: 'Мария Светлова', initials: 'МС', modules: ['Видео'], price: 249, nextPayment: '22.02.2026', status: 'active' },
    { id: '3', author: 'Дмитрий Волков', initials: 'ДВ', modules: ['Музыка'], price: 199, nextPayment: '01.03.2026', status: 'active' },
  ];

  const library = [
    { id: '1', title: 'Ночной город', author: 'Алексей Морозов', type: 'audio', format: 'FLAC', duration: '4:23' },
    { id: '2', title: 'Рассвет', author: 'Алексей Морозов', type: 'audio', format: 'WAV', duration: '3:45' },
    { id: '3', title: 'Making of: Сигналы', author: 'Алексей Морозов', type: 'video', format: 'MP4', duration: '12:34' },
    { id: '4', title: 'Пульс', author: 'Алексей Морозов', type: 'audio', format: 'FLAC', duration: '5:12' },
    { id: '5', title: 'Digital Art Process', author: 'Мария Светлова', type: 'video', format: 'MP4', duration: '18:45' },
    { id: '6', title: 'Новый альбом (demo)', author: 'Дмитрий Волков', type: 'audio', format: 'FLAC', duration: '4:01' },
    { id: '7', title: 'Live Session', author: 'Алексей Морозов', type: 'video', format: 'MP4', duration: '45:20' },
    { id: '8', title: 'Тишина', author: 'Алексей Морозов', type: 'audio', format: 'FLAC', duration: '6:01' },
  ];

  const payments = [
    { id: '1', date: '15.01.2026', author: 'Алексей Морозов', amount: 349, method: 'МИР ••4523', status: 'Оплачено' },
    { id: '2', date: '10.01.2026', author: 'Мария Светлова', amount: 249, method: 'СБП', status: 'Оплачено' },
    { id: '3', date: '05.01.2026', author: 'Дмитрий Волков', amount: 199, method: 'МИР ••4523', status: 'Оплачено' },
    { id: '4', date: '15.12.2025', author: 'Алексей Морозов', amount: 349, method: 'МИР ••4523', status: 'Оплачено' },
  ];

  const tabs: { id: Tab; label: string; icon: ReactNode }[] = [
    { id: 'library', label: 'Библиотека', icon: <BookOpen className="w-4 h-4" /> },
    { id: 'subscriptions', label: 'Подписки', icon: <Heart className="w-4 h-4" /> },
    { id: 'payments', label: 'Платежи', icon: <CreditCard className="w-4 h-4" /> },
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
          {activeTab === 'library' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Моя библиотека</h1>
              <div className="space-y-2">
                {library.map(item => (
                  <div key={item.id} className="surface p-4 flex items-center gap-4 group hover:border-ink-border-strong transition">
                    <div className={`w-10 h-10 flex items-center justify-center text-xs font-mono ${item.type === 'audio' ? 'bg-copper/10 text-copper' : 'bg-info/10 text-info'}`}>
                      {item.type === 'audio' ? '♪' : '▶'}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="text-sm font-medium truncate">{item.title}</h3>
                        <span className="tag text-[10px]">{item.format}</span>
                      </div>
                      <div className="flex items-center gap-3 text-xs text-paper-muted">
                        <span>{item.author}</span>
                        <span className="font-mono">{item.duration}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'subscriptions' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Подписки</h1>
              <div className="space-y-4">
                {subscriptions.map(sub => (
                  <div key={sub.id} className="surface p-6">
                    <div className="flex items-center justify-between mb-4">
                      <div className="flex items-center gap-4">
                        <div className="w-12 h-12 rounded-full bg-ink-surface border border-ink-border-strong flex items-center justify-center font-mono text-sm text-copper">
                          {sub.initials}
                        </div>
                        <div>
                          <h3 className="font-medium">{sub.author}</h3>
                          <div className="flex items-center gap-2 mt-1">
                            {sub.modules.map((m, i) => (
                              <span key={i} className="tag text-[10px]">{m}</span>
                            ))}
                          </div>
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="font-display text-xl font-semibold num-display">{sub.price} ₽<span className="text-sm text-paper-muted font-normal">/мес</span></div>
                        <div className="text-xs text-paper-muted font-mono mt-1">След. списание: {sub.nextPayment}</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Link to={`/${sub.author.toLowerCase().replace(/\s/g, '-')}`} className="btn-ghost px-4 py-2 rounded text-xs">Перейти</Link>
                      <button className="btn-ghost px-4 py-2 rounded text-xs">Управлять</button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'payments' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">История платежей</h1>
              <div className="surface">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-ink-border">
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Дата</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Автор</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Сумма</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Способ</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Статус</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-ink-border">
                    {payments.map(p => (
                      <tr key={p.id} className="hover:bg-ink-elevated transition">
                        <td className="px-5 py-3 text-sm font-mono">{p.date}</td>
                        <td className="px-5 py-3 text-sm">{p.author}</td>
                        <td className="px-5 py-3 text-sm font-mono num-display">{p.amount} ₽</td>
                        <td className="px-5 py-3 text-sm text-paper-muted">{p.method}</td>
                        <td className="px-5 py-3"><span className="tag text-[10px] border-success/30 text-success">{p.status}</span></td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {activeTab === 'settings' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Настройки</h1>
              <div className="surface p-6 space-y-6 max-w-2xl">
                <div>
                  <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Email</label>
                  <input type="email" defaultValue="user@example.com" className="input w-full px-4 py-3 rounded text-sm" />
                </div>
                <div>
                  <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Имя</label>
                  <input type="text" defaultValue="Иван Петров" className="input w-full px-4 py-3 rounded text-sm" />
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
