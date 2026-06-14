interface StaleBannerProps {
  stale: boolean;
}

export function StaleBanner({ stale }: StaleBannerProps) {
  if (!stale) return null;

  return (
    <div
      style={{
        backgroundColor: '#fef3c7',
        color: '#92400e',
        padding: '0.5rem 1rem',
        textAlign: 'center',
        fontSize: '0.875rem',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '1rem',
      }}
      role="alert"
    >
      <span>New version available — reload</span>
      <button
        className="btn"
        style={{ padding: '0.25rem 0.75rem', fontSize: '0.875rem' }}
        onClick={() => window.location.reload()}
      >
        Reload
      </button>
    </div>
  );
}
