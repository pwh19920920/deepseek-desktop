import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface DshStatusEvent {
  status: 'loading' | 'ready' | 'restarting' | 'error';
  phase?: string;
  message: string;
}

const PHASE_ORDER = ['init', 'profile', 'runtime', 'sidecar'];

function getPhaseIndex(phase?: string): number {
  if (!phase) return 0;
  const index = PHASE_ORDER.indexOf(phase);
  return index >= 0 ? index : 0;
}

function getPhaseLabel(phase?: string): string {
  switch (phase) {
    case 'init':
      return '初始化';
    case 'profile':
      return '检查配置';
    case 'runtime':
      return '准备运行时';
    case 'sidecar':
      return '启动服务';
    default:
      return '启动中';
  }
}

export default function App() {
  const [status, setStatus] = useState<DshStatusEvent>({
    status: 'loading',
    phase: 'init',
    message: '正在初始化…',
  });

  useEffect(() => {
    const unlisten = listen<DshStatusEvent>('dsh-status', (event) => {
      setStatus(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const phaseIndex = getPhaseIndex(status.phase);
  const progress = ((phaseIndex + 1) / PHASE_ORDER.length) * 100;

  if (status.status === 'error') {
    return (
      <div className="container">
        <div className="logo">
          <div
            className="logo-ring"
            style={{ borderTopColor: '#ef4444', borderRightColor: '#f87171' }}
          />
          <div
            className="logo-ring"
            style={{
              borderTopColor: '#f87171',
              borderBottomColor: '#ef4444',
            }}
          />
          <div className="logo-inner" style={{ color: '#ef4444' }}>
            !
          </div>
        </div>
        <h1
          style={{
            background: 'none',
            WebkitTextFillColor: '#ef4444',
            color: '#ef4444',
          }}
        >
          启动失败
        </h1>
        <p className="error-detail">{status.message}</p>
      </div>
    );
  }

  if (status.status === 'restarting') {
    return (
      <div className="container">
        <div className="logo">
          <div
            className="logo-ring"
            style={{ borderTopColor: '#3b82f6', borderRightColor: '#60a5fa' }}
          />
          <div
            className="logo-ring"
            style={{
              borderTopColor: '#60a5fa',
              borderBottomColor: '#3b82f6',
            }}
          />
          <div className="logo-inner" style={{ color: '#3b82f6' }}>↻</div>
        </div>
        <h1>正在重启</h1>
        <p>{status.message}</p>
        <div className="loading-bar">
          <div className="loading-bar-fill"></div>
        </div>
      </div>
    );
  }

  return (
    <div className="container">
      <div className="logo">
        <div className="logo-ring"></div>
        <div className="logo-ring"></div>
        <div className="logo-inner">DS</div>
      </div>
      <h1>DeepSeek Harness</h1>
      <p className="phase-label">{getPhaseLabel(status.phase)}</p>
      <div className="progress-container">
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="progress-steps">
          {PHASE_ORDER.map((phase, index) => (
            <div
              key={phase}
              className={`progress-step ${
                index <= phaseIndex ? 'active' : ''
              }`}
            >
              <div
                className={`step-dot ${
                  index <= phaseIndex ? 'active' : ''
                }`}
              />
              <span className="step-label">
                {getPhaseLabel(phase)}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}