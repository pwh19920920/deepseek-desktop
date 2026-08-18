import { useState, useEffect } from 'react';

export default function App() {
  const [status, setStatus] = useState<'loading' | 'error'>('loading');
  const [error, setError] = useState('');

  useEffect(() => {
    // Timeout after 30s to show error
    const timer = setTimeout(() => {
      if (status === 'loading') {
        setStatus('error');
        setError('Failed to connect to harness sidecar.');
      }
    }, 30000);
    return () => clearTimeout(timer);
  }, [status]);

  if (status === 'error') {
    return (
      <div className="container">
        <div className="logo">⚠️</div>
        <h1 className="error">Failed to Start</h1>
        <p>{error}</p>
      </div>
    );
  }

  return (
    <div className="container">
      <div className="logo">⚡</div>
      <h1>DeepSeek Harness</h1>
      <div className="spinner"></div>
      <p>Starting harness sidecar...</p>
    </div>
  );
}
