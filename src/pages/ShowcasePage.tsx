import { useState, useRef, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Play, Pause, Lock, Music, Video, ExternalLink } from 'lucide-react';

const AUTHOR = {
  name: 'Алексей Морозов',
  slug: 'aleksei-morozov',
  initials: 'АМ',
  description: 'Электронный музыкант и продюсер. Эксклюзивные треки в Lossless качестве, которых нет на других площадках.',
  socials: { telegram: '@morozov_music', vk: 'morozov_music', youtube: 'AlekseiMorozov' },
  subscribers: 1247,
  verified: true,
};

const TRACKS = [
  { id: 1, title: 'Ночной город', album: 'Сигналы', duration: '4:23', type: 'audio' as const, format: 'FLAC', year: '2025' },
  { id: 2, title: 'Рассвет', album: 'Сигналы', duration: '3:45', type: 'audio' as const, format: 'WAV', year: '2025' },
  { id: 3, title: 'Пульс', album: 'Импульс', duration: '5:12', type: 'audio' as const, format: 'FLAC', year: '2024' },
  { id: 4, title: 'Тишина', album: 'Импульс', duration: '6:01', type: 'audio' as const, format: 'FLAC', year: '2024' },
  { id: 5, title: 'Эхо', album: 'Отражения', duration: '4:55', type: 'audio' as const, format: 'MP3 320', year: '2023' },
];

const VIDEOS = [
  { id: 1, title: 'Making of: Сигналы', duration: '12:34', type: 'video' as const, year: '2025' },
  { id: 2, title: 'Live Session (закрытый концерт)', duration: '45:20', type: 'video' as const, year: '2024' },
  { id: 3, title: 'Studio Vlog #3', duration: '8:15', type: 'video' as const, year: '2024' },
];

const PLANS = [
  { id: 'music', title: 'Музыка', price: 199, period: 'мес', description: 'Все треки в Lossless', features: ['FLAC · WAV · MP3 320', 'Все релизы', 'Ранний доступ'] },
  { id: 'video', title: 'Видео', price: 249, period: 'мес', description: 'Эксклюзивные видео', features: ['HD · 4K качество', 'Behind the scenes', 'Live записи'] },
  { id: 'combo', title: 'Комбо', price: 349, period: 'мес', description: 'Музыка + Видео', features: ['Всё из Музыка', 'Всё из Видео', 'Экономия 20%'], popular: true },
];

export default function ShowcasePage() {
  const [activeTab, setActiveTab] = useState<'all' | 'audio' | 'video'>('all');
  const [playingTrack, setPlayingTrack] = useState<number | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [teaserProgress, setTeaserProgress] = useState(0);
  const [showPaywall, setShowPaywall] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (isPlaying && playingTrack) {
      intervalRef.current = setInterval(() => {
        setTeaserProgress(prev => {
          if (prev >= 100) {
            setIsPlaying(false);
            setShowPaywall(true);
            return 100;
          }
          return prev + (100 / 15);
        });
      }, 1000);
    }
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [isPlaying, playingTrack]);

  const handlePlay = (trackId: number) => {
    if (playingTrack === trackId) {
      setIsPlaying(!isPlaying);
    } else {
      setPlayingTrack(trackId);
      setTeaserProgress(0);
      setIsPlaying(true);
      setShowPaywall(false);
    }
  };

  const filteredTracks = activeTab === 'all' ? [...TRACKS, ...VIDEOS] :
    activeTab === 'audio' ? TRACKS : VIDEOS;

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
          <Link to="/login" className="btn-primary px-4 py-2 text-sm rounded">Подписаться</Link>
        </div>
      </nav>

      {/* Author Header */}
      <section className="pt-32 pb-12 px-6 border-b border-ink-border">
        <div className="max-w-[1400px] mx-auto">
          <div className="flex flex-col md:flex-row items-start gap-8">
            <div className="w-32 h-32 bg-ink-surface border border-ink-border-strong flex items-center justify-center font-display text-4xl font-semibold text-copper">
              {AUTHOR.initials}
            </div>
            <div className="flex-1">
              <div className="flex items-center gap-3 mb-3">
                <h1 className="font-display text-4xl md:text-5xl font-semibold tracking-tight">{AUTHOR.name}</h1>
                {AUTHOR.verified && (
                  <div className="tag tag-copper">
                    <div className="w-1.5 h-1.5 rounded-full bg-copper" />
                    Verified
                  </div>
                )}
              </div>
              <p className="text-lg text-paper-muted mb-6 max-w-2xl">{AUTHOR.description}</p>
              <div className="flex items-center gap-6 text-sm">
                <div className="font-mono text-paper-dim">
                  <span className="text-paper num-display">{AUTHOR.subscribers.toLocaleString()}</span> подписчиков
                </div>
                <div className="flex items-center gap-4">
                  <a href="#" className="text-paper-muted hover:text-copper transition">Telegram</a>
                  <a href="#" className="text-paper-muted hover:text-copper transition">VK</a>
                  <a href="#" className="text-paper-muted hover:text-copper transition">YouTube</a>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Plans */}
      <section className="py-16 px-6 border-b border-ink-border bg-ink-elevated">
        <div className="max-w-[1400px] mx-auto">
          <div className="font-mono text-xs text-copper uppercase tracking-wider mb-8">Тарифы</div>
          <div className="grid md:grid-cols-3 gap-6">
            {PLANS.map(plan => (
              <div key={plan.id} className={`surface p-8 relative ${plan.popular ? 'border-copper' : ''}`}>
                {plan.popular && (
                  <div className="absolute -top-3 left-8 tag tag-copper">Популярный</div>
                )}
                <h3 className="font-display text-2xl font-semibold mb-2">{plan.title}</h3>
                <p className="text-paper-muted mb-6">{plan.description}</p>
                <div className="mb-6">
                  <span className="font-display text-4xl font-semibold num-display">{plan.price}</span>
                  <span className="text-paper-muted ml-2">₽/{plan.period}</span>
                </div>
                <ul className="space-y-2 mb-8">
                  {plan.features.map((feature, i) => (
                    <li key={i} className="flex items-start gap-2 text-sm text-paper-muted">
                      <div className="w-1 h-1 rounded-full bg-copper mt-2 flex-shrink-0" />
                      <span>{feature}</span>
                    </li>
                  ))}
                </ul>
                <Link to="/login" className={`w-full py-3 rounded text-center font-medium transition ${plan.popular ? 'btn-primary' : 'btn-ghost'}`}>
                  Подписаться
                </Link>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Content */}
      <section className="py-16 px-6">
        <div className="max-w-[1400px] mx-auto">
          <div className="flex items-center justify-between mb-8">
            <div className="font-mono text-xs text-copper uppercase tracking-wider">Контент</div>
            <div className="flex items-center gap-2">
              {(['all', 'audio', 'video'] as const).map(tab => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`px-4 py-2 text-sm rounded transition ${activeTab === tab ? 'bg-copper text-ink' : 'text-paper-muted hover:text-paper'}`}
                >
                  {tab === 'all' ? 'Всё' : tab === 'audio' ? 'Аудио' : 'Видео'}
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-2">
            {filteredTracks.map(track => (
              <div key={`${track.type}-${track.id}`} className="surface p-4 flex items-center gap-4 group">
                <button
                  onClick={() => handlePlay(track.id)}
                  className="w-12 h-12 bg-ink border border-ink-border-strong flex items-center justify-center hover:border-copper transition"
                >
                  {playingTrack === track.id && isPlaying ? (
                    <Pause className="w-4 h-4 text-copper" />
                  ) : (
                    <Play className="w-4 h-4 text-paper-muted group-hover:text-copper transition" />
                  )}
                </button>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="font-medium truncate">{track.title}</h3>
                    {track.type === 'audio' && (
                      <span className="tag text-[10px]">{track.format}</span>
                    )}
                  </div>
                  <div className="flex items-center gap-3 text-sm text-paper-muted">
                    {track.type === 'audio' && <span>{track.album}</span>}
                    <span className="font-mono text-xs">{track.duration}</span>
                    <span className="font-mono text-xs">{track.year}</span>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  {playingTrack === track.id && (
                    <div className="w-32 h-1 bg-ink-border rounded-full overflow-hidden">
                      <div className="h-full bg-copper transition-all" style={{ width: `${teaserProgress}%` }} />
                    </div>
                  )}
                  <Lock className="w-4 h-4 text-paper-dim" />
                </div>
              </div>
            ))}
          </div>

          {showPaywall && (
            <div className="mt-8 surface p-8 text-center">
              <Lock className="w-8 h-8 text-copper mx-auto mb-4" />
              <h3 className="font-display text-xl font-semibold mb-2">Хотите слушать полностью?</h3>
              <p className="text-paper-muted mb-6">Оформите подписку и получите доступ ко всему контенту</p>
              <Link to="/login" className="btn-primary px-8 py-3 rounded inline-block">Подписаться от 199 ₽/мес</Link>
            </div>
          )}
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-ink-border py-8 px-6">
        <div className="max-w-[1400px] mx-auto flex flex-col md:flex-row justify-between items-center gap-4 text-xs text-paper-dim">
          <div>© 2026 Чистовик · {AUTHOR.name}</div>
          <div className="font-mono">Платформа закрытого контента</div>
        </div>
      </footer>
    </div>
  );
}
