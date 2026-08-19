import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface DshStatusEvent {
  status: 'loading' | 'ready' | 'error';
  message: string;
}

export default function App() {
  const [status, setStatus] = useState<DshStatusEvent>({
    status: 'loading',
    message: 'Starting harness sidecar...',
  });

  useEffect(() => {
    const unlisten = listen<DshStatusEvent>('dsh-status', (event) => {
      setStatus(event.payload);
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  if (status.status === 'error') {
    return (
      <div className="container">
        <div className="logo">
          <div className="logo-ring" style={{ borderTopColor: '#ef4444', borderRightColor: '#f87171' }}></div>
          <div className="logo-ring" style={{ borderTopColor: '#f87171', borderBottomColor: '#ef4444' }}></div>
          <div className="logo-inner" style={{ color: '#ef4444' }}>!</div>
        </div>
        <h1 style={{ background: 'none', WebkitTextFillColor: '#ef4444', color: '#ef4444' }}>Failed to Start</h1>
        <p className="error-detail">{status.message}</p>
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
      <p>{status.message}</p>
      <div className="loading-bar">
        <div className="loading-bar-fill"></div>
      </div>
    </div>
  );
}