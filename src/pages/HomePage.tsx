import { Link } from 'react-router-dom';
import { ArrowRight, Lock, Disc3, Radio } from 'lucide-react';

export default function HomePage() {
  return (
    <div className="min-h-screen bg-ink text-paper">
      {/* Nav */}
      <nav className="fixed top-0 w-full z-50 border-b border-ink-border bg-ink/80 backdrop-blur-sm">
        <div className="max-w-[1400px] mx-auto px-6 h-16 flex items-center justify-between">
          <Link to="/" className="flex items-center gap-2.5">
            <div className="w-7 h-7 border border-copper flex items-center justify-center">
              <div className="w-2 h-2 bg-copper" />
            </div>
            <span className="font-display font-semibold text-lg tracking-tight">ЧИСТОВИК</span>
          </Link>
          <div className="hidden md:flex items-center gap-8 text-sm text-paper-muted">
            <a href="#manifest" className="link-copper">Манифест</a>
            <a href="#how" className="link-copper">Как это работает</a>
            <a href="#authors" className="link-copper">Авторы</a>
          </div>
          <div className="flex items-center gap-3">
            <Link to="/login" className="text-sm text-paper-muted hover:text-paper transition">Войти</Link>
            <Link to="/login" className="btn-primary px-4 py-2 text-sm rounded">Начать</Link>
          </div>
        </div>
      </nav>

      {/* Hero */}
      <section className="relative pt-32 pb-24 px-6 overflow-hidden noise">
        <div className="absolute inset-0 grid-bg opacity-40 fade-mask" />
        <div className="max-w-[1400px] mx-auto relative">
          <div className="max-w-4xl">
            <div className="flex items-center gap-2 mb-8">
              <div className="w-1.5 h-1.5 rounded-full bg-copper animate-pulse" />
              <span className="font-mono text-xs text-paper-muted uppercase tracking-wider">v1.0 · Закрытая платформа</span>
            </div>
            <h1 className="font-display text-6xl md:text-8xl font-semibold leading-[0.9] tracking-tighter mb-8">
              Контент<br />
              <span className="text-copper">без компромиссов.</span>
            </h1>
            <p className="text-xl md:text-2xl text-paper-muted max-w-2xl leading-relaxed mb-12">
              Платформа для авторов, которые не готовы снижать качество. Lossless-аудио, защищённое видео, прямые подписки — без посредников и алгоритмов.
            </p>
            <div className="flex flex-col sm:flex-row gap-4">
              <Link to="/login" className="btn-primary px-8 py-4 rounded inline-flex items-center justify-center gap-2 text-base">
                Создать витрину <ArrowRight className="w-4 h-4" />
              </Link>
              <Link to="/aleksei-morozov" className="btn-ghost px-8 py-4 rounded inline-flex items-center justify-center gap-2 text-base">
                Посмотреть пример
              </Link>
            </div>
          </div>

          {/* Stats strip */}
          <div className="mt-24 grid grid-cols-2 md:grid-cols-4 gap-px bg-ink-border border border-ink-border">
            {[
              { label: 'Lossless', value: 'FLAC · WAV', sub: 'Без потерь' },
              { label: 'Защита', value: 'HLS + DRM', sub: 'AES-128' },
              { label: 'Комиссия', value: '8%', sub: 'Прозрачно' },
              { label: 'Выплаты', value: 'T+7', sub: 'На счёт' },
            ].map((stat, i) => (
              <div key={i} className="bg-ink p-6">
                <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-2">{stat.label}</div>
                <div className="font-display text-2xl font-semibold mb-1">{stat.value}</div>
                <div className="text-sm text-paper-muted">{stat.sub}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Manifest */}
      <section id="manifest" className="py-32 px-6 border-t border-ink-border">
        <div className="max-w-[1400px] mx-auto">
          <div className="grid md:grid-cols-12 gap-12">
            <div className="md:col-span-4">
              <div className="font-mono text-xs text-copper uppercase tracking-wider mb-4">01 — Манифест</div>
              <h2 className="font-display text-4xl md:text-5xl font-semibold leading-tight tracking-tight">
                Мы верим в качество.
              </h2>
            </div>
            <div className="md:col-span-7 md:col-start-6 space-y-6 text-lg text-paper-muted leading-relaxed">
              <p>
                Стриминги сжали музыку до 256 kbps. Алгоритмы решают, что вы слушаете. Пиратство убивает независимых артистов.
              </p>
              <p>
                <span className="text-paper">Чистовик</span> — это альтернатива. Платформа, где автор контролирует всё: от качества мастеринга до цены подписки. Где слушатель получает настоящий Lossless, а не "потеряшку".
              </p>
              <p>
                Мы не гонимся за охватом. Мы делаем инструмент для тех, кому важно <span className="text-copper">каждое децибел</span>.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* How it works */}
      <section id="how" className="py-32 px-6 border-t border-ink-border bg-ink-elevated">
        <div className="max-w-[1400px] mx-auto">
          <div className="font-mono text-xs text-copper uppercase tracking-wider mb-4">02 — Как это работает</div>
          <h2 className="font-display text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-16 max-w-2xl">
            Три шага до первого рубля.
          </h2>

          <div className="grid md:grid-cols-3 gap-px bg-ink-border border border-ink-border">
            {[
              {
                num: '01',
                icon: <Disc3 className="w-5 h-5" />,
                title: 'Загрузите',
                desc: 'FLAC, WAV, MP4. Нарезайте тизеры — 15 сек аудио, 60 сек видео. Контент шифруется и уходит в защищённое хранилище.',
              },
              {
                num: '02',
                icon: <Lock className="w-5 h-5" />,
                title: 'Защитите',
                desc: 'HLS-стриминг с AES-128. Водяные знаки с ID пользователя. Лимит сессий. Токены живут 2 часа.',
              },
              {
                num: '03',
                icon: <Radio className="w-5 h-5" />,
                title: 'Монетизируйте',
                desc: 'Модульные подписки: музыка, видео, комбо. Автопродление через ЮKassa. Выплаты на счёт каждые 7 дней.',
              },
            ].map((step, i) => (
              <div key={i} className="bg-ink-elevated p-8">
                <div className="flex items-center justify-between mb-6">
                  <span className="font-mono text-sm text-paper-dim">{step.num}</span>
                  <div className="w-8 h-8 border border-ink-border-strong flex items-center justify-center text-copper">
                    {step.icon}
                  </div>
                </div>
                <h3 className="font-display text-2xl font-semibold mb-3">{step.title}</h3>
                <p className="text-paper-muted leading-relaxed">{step.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features grid */}
      <section className="py-32 px-6 border-t border-ink-border">
        <div className="max-w-[1400px] mx-auto">
          <div className="font-mono text-xs text-copper uppercase tracking-wider mb-4">03 — Возможности</div>
          <h2 className="font-display text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-16 max-w-2xl">
            Всё, что нужно автору.
          </h2>

          <div className="grid md:grid-cols-2 gap-6">
            {[
              {
                title: 'Lossless-аудио',
                items: ['FLAC, WAV, MP3 320', 'Потоковое воспроизведение', 'Метаданные и обложки', 'Альбомы и синглы'],
              },
              {
                title: 'Защищённое видео',
                items: ['HLS с шифрованием', 'Адаптивное качество (360p–4K)', 'Водяные знаки в реальном времени', 'Запрет скачивания'],
              },
              {
                title: 'Модульные подписки',
                items: ['Музыка / Видео / Комбо', 'Месяц / квартал / год', 'Пропорциональный перерасчёт', 'Заморозка до 3 месяцев'],
              },
              {
                title: 'Аналитика и выплаты',
                items: ['Дашборд в реальном времени', 'Конверсия тизер → подписка', 'Выплаты на р/с или карту', 'Прозрачная комиссия 8%'],
              },
            ].map((feature, i) => (
              <div key={i} className="surface p-8">
                <h3 className="font-display text-xl font-semibold mb-5">{feature.title}</h3>
                <ul className="space-y-3">
                  {feature.items.map((item, j) => (
                    <li key={j} className="flex items-start gap-3 text-paper-muted">
                      <div className="w-1 h-1 rounded-full bg-copper mt-2 flex-shrink-0" />
                      <span>{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Authors */}
      <section id="authors" className="py-32 px-6 border-t border-ink-border bg-ink-elevated">
        <div className="max-w-[1400px] mx-auto">
          <div className="font-mono text-xs text-copper uppercase tracking-wider mb-4">04 — Авторы</div>
          <h2 className="font-display text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-16 max-w-2xl">
            Платформа растёт.
          </h2>

          <div className="grid md:grid-cols-4 gap-6">
            {[
              { name: 'Алексей Морозов', type: 'Электронная музыка', subs: '1.2K', avatar: 'AM' },
              { name: 'Мария Светлова', type: 'Видео-арт', subs: '892', avatar: 'МС' },
              { name: 'Дмитрий Волков', type: 'Инди-рок', subs: '2.1K', avatar: 'ДВ' },
              { name: 'Анна Козлова', type: 'Мастер-классы', subs: '567', avatar: 'АК' },
            ].map((author, i) => (
              <Link to="/aleksei-morozov" key={i} className="surface p-6 group cursor-pointer">
                <div className="w-12 h-12 rounded-full bg-ink border border-ink-border-strong flex items-center justify-center font-mono text-sm text-copper mb-4">
                  {author.avatar}
                </div>
                <h3 className="font-display text-lg font-semibold mb-1 group-hover:text-copper transition">{author.name}</h3>
                <p className="text-sm text-paper-muted mb-3">{author.type}</p>
                <div className="font-mono text-xs text-paper-dim">{author.subs} подписчиков</div>
              </Link>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-32 px-6 border-t border-ink-border">
        <div className="max-w-[1400px] mx-auto text-center">
          <h2 className="font-display text-5xl md:text-7xl font-semibold leading-[0.9] tracking-tighter mb-8">
            Готовы к<br />
            <span className="text-copper">чистовику?</span>
          </h2>
          <p className="text-xl text-paper-muted mb-12 max-w-2xl mx-auto">
            Создайте витрину за 5 минут. Без ежемесячной платы — только комиссия с продаж.
          </p>
          <Link to="/login" className="btn-primary px-10 py-5 rounded inline-flex items-center gap-2 text-lg">
            Начать бесплатно <ArrowRight className="w-5 h-5" />
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-ink-border py-12 px-6">
        <div className="max-w-[1400px] mx-auto">
          <div className="grid md:grid-cols-4 gap-8 mb-12">
            <div>
              <div className="flex items-center gap-2.5 mb-4">
                <div className="w-7 h-7 border border-copper flex items-center justify-center">
                  <div className="w-2 h-2 bg-copper" />
                </div>
                <span className="font-display font-semibold">ЧИСТОВИК</span>
              </div>
              <p className="text-sm text-paper-muted">Платформа закрытого контента для авторов и их аудитории.</p>
            </div>
            <div>
              <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-4">Платформа</div>
              <ul className="space-y-2 text-sm">
                <li><a href="#" className="text-paper-muted hover:text-paper transition">О нас</a></li>
                <li><a href="#" className="text-paper-muted hover:text-paper transition">Блог</a></li>
                <li><a href="#" className="text-paper-muted hover:text-paper transition">Контакты</a></li>
              </ul>
            </div>
            <div>
              <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-4">Авторам</div>
              <ul className="space-y-2 text-sm">
                <li><a href="#" className="text-paper-muted hover:text-paper transition">Как начать</a></li>
                <li><a href="#" className="text-paper-muted hover:text-paper transition">Тарифы</a></li>
                <li><a href="#" className="text-paper-muted hover:text-paper transition">FAQ</a></li>
              </ul>
            </div>
            <div>
              <div className="font-mono text-xs text-paper-dim uppercase tracking-wider mb-4">Правовое</div>
              <ul className="space-y-2 text-sm">
                <li><a href="#" className="text-paper-muted hover:text-paper transition">Оферта</a></li>
                <li><a href="#" className="text-paper-muted hover:text-paper transition">Конфиденциальность</a></li>
                <li><a href="#" className="text-paper-muted hover:text-paper transition">152-ФЗ</a></li>
              </ul>
            </div>
          </div>
          <div className="divider mb-6" />
          <div className="flex flex-col md:flex-row justify-between items-center gap-4 text-xs text-paper-dim">
            <div>© 2026 Чистовик. Все права защищены.</div>
            <div className="font-mono">Сделано в России · v1.0.0</div>
          </div>
        </div>
      </footer>
    </div>
  );
}
