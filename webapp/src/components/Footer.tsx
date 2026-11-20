import { useEffect, useState } from 'react';
import SiteService from 'services/SiteService';
export const Footer: React.FC = () => {
  const [title, setTitle] = useState('Photolio');
  useEffect(() => { SiteService.getPublicSite().then((site) => setTitle(site.title)).catch(() => undefined); }, []);
  return <footer style={{ width: '100%', textAlign: 'center', padding: '18px 0 6px', opacity: .7 }}>
    <small>© {new Date().getFullYear()} {title} · Powered by Photolio</small>
  </footer>;
};