import { useState, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { LayoutDashboard, FileAudio, DollarSign, BarChart3, Wallet, Settings, Plus, ExternalLink } from 'lucide-react';

type Tab = 'overview' | 'content' | 'pricing' | 'stats' | 'payouts' | 'settings';

export default function AuthorDashboard() {
  const [activeTab, setActiveTab] = useState<Tab>('overview');

  const stats = [
    { label: 'Подписчики', value: '1,247', change: '+24' },
    { label: 'Доход (мес)', value: '287K ₽', change: '+18%' },
    { label: 'Конверсия', value: '12.4%', change: '+2.1' },
    { label: 'Прослушивания', value: '45.8K', change: '+31%' },
  ];

  const content = [
    { id: 1, title: 'Ночной город', type: 'audio', format: 'FLAC', plays: 3420, status: 'published' },
    { id: 2, title: 'Рассвет', type: 'audio', format: 'WAV', plays: 2180, status: 'published' },
    { id: 3, title: 'Making of: Сигналы', type: 'video', format: 'MP4', plays: 1540, status: 'published' },
    { id: 4, title: 'Пульс', type: 'audio', format: 'FLAC', plays: 980, status: 'draft' },
    { id: 5, title: 'Live Session', type: 'video', format: 'MOV', plays: 0, status: 'draft' },
  ];

  const tabs: { id: Tab; label: string; icon: ReactNode }[] = [
    { id: 'overview', label: 'Обзор', icon: <LayoutDashboard className="w-4 h-4" /> },
    { id: 'content', label: 'Контент', icon: <FileAudio className="w-4 h-4" /> },
    { id: 'pricing', label: 'Тарифы', icon: <DollarSign className="w-4 h-4" /> },
    { id: 'stats', label: 'Статистика', icon: <BarChart3 className="w-4 h-4" /> },
    { id: 'payouts', label: 'Выплаты', icon: <Wallet className="w-4 h-4" /> },
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

        <div className="p-3 border-t border-ink-border">
          <Link to="/aleksei-morozov" className="flex items-center gap-2 px-3 py-2 text-sm text-paper-muted hover:text-paper transition">
            <ExternalLink className="w-3.5 h-3.5" />
            Моя витрина
          </Link>
        </div>
      </aside>

      {/* Main */}
      <main className="flex-1 ml-60">
        {/* Top bar */}
        <div className="h-14 border-b border-ink-border flex items-center justify-between px-6">
          <div className="font-mono text-xs text-paper-dim uppercase tracking-wider">
            {tabs.find(t => t.id === activeTab)?.label}
          </div>
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-full bg-ink-surface border border-ink-border-strong flex items-center justify-center font-mono text-xs text-copper">АМ</div>
          </div>
        </div>

        <div className="p-6">
          {activeTab === 'overview' && (
            <div>
              <div className="flex items-center justify-between mb-8">
                <h1 className="font-display text-3xl font-semibold">Добро пожаловать, Алексей</h1>
                <button className="btn-primary px-4 py-2 rounded text-sm flex items-center gap-2">
                  <Plus className="w-4 h-4" />
                  Загрузить
                </button>
              </div>

              {/* Stats grid */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-px bg-ink-border border border-ink-border mb-8">
                {stats.map((stat, i) => (
                  <div key={i} className="bg-ink p-5">
                    <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-2">{stat.label}</div>
                    <div className="flex items-baseline gap-2">
                      <span className="font-display text-2xl font-semibold num-display">{stat.value}</span>
                      <span className="text-xs text-success font-mono">{stat.change}</span>
                    </div>
                  </div>
                ))}
              </div>

              {/* Recent content */}
              <div className="surface">
                <div className="px-5 py-4 border-b border-ink-border flex items-center justify-between">
                  <h2 className="font-medium">Последний контент</h2>
                  <button onClick={() => setActiveTab('content')} className="text-sm text-copper hover:underline">Все →</button>
                </div>
                <div className="divide-y divide-ink-border">
                  {content.slice(0, 5).map(item => (
                    <div key={item.id} className="px-5 py-3 flex items-center justify-between hover:bg-ink-elevated transition">
                      <div className="flex items-center gap-3">
                        <div className={`w-8 h-8 flex items-center justify-center text-xs font-mono ${item.type === 'audio' ? 'bg-copper/10 text-copper' : 'bg-info/10 text-info'}`}>
                          {item.type === 'audio' ? '♪' : '▶'}
                        </div>
                        <div>
                          <div className="text-sm font-medium">{item.title}</div>
                          <div className="text-xs text-paper-muted font-mono">{item.format} · {item.plays.toLocaleString()} прослушиваний</div>
                        </div>
                      </div>
                      <div className={`tag text-[10px] ${item.status === 'published' ? 'border-success/30 text-success' : ''}`}>
                        {item.status === 'published' ? 'Live' : 'Draft'}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === 'content' && (
            <div>
              <div className="flex items-center justify-between mb-8">
                <h1 className="font-display text-3xl font-semibold">Контент</h1>
                <button className="btn-primary px-4 py-2 rounded text-sm flex items-center gap-2">
                  <Plus className="w-4 h-4" />
                  Загрузить
                </button>
              </div>

              <div className="surface">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-ink-border">
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Название</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Тип</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Формат</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Прослушивания</th>
                      <th className="text-left px-5 py-3 text-xs font-mono text-paper-dim uppercase tracking-wider">Статус</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-ink-border">
                    {content.map(item => (
                      <tr key={item.id} className="hover:bg-ink-elevated transition">
                        <td className="px-5 py-3 text-sm font-medium">{item.title}</td>
                        <td className="px-5 py-3 text-sm text-paper-muted capitalize">{item.type === 'audio' ? 'Аудио' : 'Видео'}</td>
                        <td className="px-5 py-3"><span className="tag text-[10px]">{item.format}</span></td>
                        <td className="px-5 py-3 text-sm font-mono num-display">{item.plays.toLocaleString()}</td>
                        <td className="px-5 py-3">
                          <span className={`tag text-[10px] ${item.status === 'published' ? 'border-success/30 text-success' : ''}`}>
                            {item.status === 'published' ? 'Live' : 'Draft'}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {activeTab === 'pricing' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Тарифы</h1>
              <div className="grid md:grid-cols-3 gap-6">
                {[
                  { title: 'Музыка', price: 199, subs: 892, enabled: true },
                  { title: 'Видео', price: 249, subs: 534, enabled: true },
                  { title: 'Комбо', price: 349, subs: 355, enabled: true },
                ].map((plan, i) => (
                  <div key={i} className="surface p-6">
                    <div className="flex items-center justify-between mb-4">
                      <h3 className="font-display text-xl font-semibold">{plan.title}</h3>
                      <div className={`w-2 h-2 rounded-full ${plan.enabled ? 'bg-success' : 'bg-paper-dim'}`} />
                    </div>
                    <div className="mb-4">
                      <span className="font-display text-3xl font-semibold num-display">{plan.price}</span>
                      <span className="text-paper-muted ml-1">₽/мес</span>
                    </div>
                    <div className="text-sm text-paper-muted mb-6">{plan.subs} подписчиков</div>
                    <button className="btn-ghost w-full py-2 rounded text-sm">Редактировать</button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'stats' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Статистика</h1>
              <div className="surface p-8">
                <div className="text-center text-paper-muted">Графики и аналитика будут доступны после подключения метрик</div>
              </div>
            </div>
          )}

          {activeTab === 'payouts' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Выплаты</h1>
              <div className="grid md:grid-cols-3 gap-6 mb-8">
                <div className="surface p-6">
                  <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-2">Доступно</div>
                  <div className="font-display text-3xl font-semibold num-display">142K ₽</div>
                </div>
                <div className="surface p-6">
                  <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-2">В обработке</div>
                  <div className="font-display text-3xl font-semibold num-display">28K ₽</div>
                </div>
                <div className="surface p-6">
                  <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-2">Выплачено всего</div>
                  <div className="font-display text-3xl font-semibold num-display">1.2M ₽</div>
                </div>
              </div>
              <button className="btn-primary px-6 py-3 rounded text-sm">Запросить выплату</button>
            </div>
          )}

          {activeTab === 'settings' && (
            <div>
              <h1 className="font-display text-3xl font-semibold mb-8">Настройки</h1>
              <div className="surface p-6 space-y-6 max-w-2xl">
                <div>
                  <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Имя автора</label>
                  <input type="text" defaultValue="Алексей Морозов" className="input w-full px-4 py-3 rounded text-sm" />
                </div>
                <div>
                  <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Slug витрины</label>
                  <input type="text" defaultValue="aleksei-morozov" className="input w-full px-4 py-3 rounded text-sm font-mono" />
                </div>
                <div>
                  <label className="block text-xs font-mono text-paper-dim uppercase tracking-wider mb-2">Описание</label>
                  <textarea defaultValue="Электронный музыкант и продюсер." className="input w-full px-4 py-3 rounded text-sm h-24 resize-none" />
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
