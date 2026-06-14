import { useEffect, useState } from 'react';

interface BuildInfo {
  build_id: string;
  release_tag: string;
  build_timestamp: string;
}

export function useStaleDetection() {
  const [stale, setStale] = useState(false);
  const [serverInfo, setServerInfo] = useState<BuildInfo | null>(null);

  useEffect(() => {
    const clientBuildId = __BUILD_ID__;
    if (clientBuildId === 'dev') return;

    fetch('/api/v1/meta/build-info', { cache: 'no-store' })
      .then(r => r.json())
      .then((info: BuildInfo) => {
        setServerInfo(info);
        if (info.build_id !== clientBuildId) {
          setStale(true);
        }
      })
      .catch(() => {/* silent fail — non-blocking */});
  }, []);

  return { stale, serverInfo };
}
